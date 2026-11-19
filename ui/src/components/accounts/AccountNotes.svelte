<script module lang="ts">
  const drafts = new Map<string, string>();
</script>

<script lang="ts">
  import { onDestroy, onMount, untrack } from "svelte";
  import { Editor, type ChainedCommands } from "@tiptap/core";
  import Paragraph from "@tiptap/extension-paragraph";
  import StarterKit from "@tiptap/starter-kit";
  import { Color, TextStyle } from "@tiptap/extension-text-style";
  import { Placeholder } from "@tiptap/extensions";
  import { t } from "../../lib/i18n/i18n.svelte";
  import {
    MAX_NOTE_BYTES,
    NOTE_COLORS,
    noteBytes,
    sanitizeNote,
    toHex,
  } from "../../lib/notes/notes";
  import { run } from "../../lib/state/state.svelte";
  import { Bold, Italic, List, RemoveFormatting, Strikethrough, Underline } from "@lucide/svelte";

  type AccountNotesPropsType = { steamId: string; note: string | null };

  let { steamId, note }: AccountNotesPropsType = $props();

  const SAVE_DELAY_MS = 700;
  const FORMATS = [
    {
      name: "bold",
      icon: Bold,
      label: () => t.notes_bold(),
      apply: (chain: ChainedCommands) => chain.toggleBold(),
    },
    {
      name: "italic",
      icon: Italic,
      label: () => t.notes_italic(),
      apply: (chain: ChainedCommands) => chain.toggleItalic(),
    },
    {
      name: "underline",
      icon: Underline,
      label: () => t.notes_underline(),
      apply: (chain: ChainedCommands) => chain.toggleUnderline(),
    },
    {
      name: "strike",
      icon: Strikethrough,
      label: () => t.notes_strikethrough(),
      apply: (chain: ChainedCommands) => chain.toggleStrike(),
    },
    {
      name: "bulletList",
      icon: List,
      label: () => t.notes_list(),
      apply: (chain: ChainedCommands) => chain.toggleBulletList(),
    },
  ] as const;

  let host = $state<HTMLDivElement | null>(null);
  let editor = $state.raw<Editor | null>(null);
  let revision = $state(0);
  let tooLong = $state(false);
  let focused = $state(false);
  let height = $state<number | null>(null);
  let timer: ReturnType<typeof setTimeout> | null = null;
  // ! Saves run one after another: two in flight could reach the database in the wrong order.
  let queue: Promise<void> = Promise.resolve();
  let pending = 0;
  let queued: string | null = null;
  let failed: string | null = null;
  let loaded = { steamId: "", note: "", html: "" };

  const read = (): string =>
    editor && editor.getText().trim() ? sanitizeNote(editor.getHTML()) : "";

  const sync = (): void => {
    if (!editor || editor.isDestroyed) return;
    const html = sanitizeNote(note);
    const sameAccount = loaded.steamId === steamId;
    const editing = editor.isFocused || timer !== null || pending > 0;
    if (sameAccount && (html === loaded.note || editing)) return;
    if (!sameAccount) save();
    editor.commands.setContent(html, { emitUpdate: false });
    loaded = { steamId, note: html, html: read() };
    queued = null;
    failed = null;
    tooLong = false;
    const draft = drafts.get(steamId);
    if (draft !== undefined && draft !== loaded.html) {
      editor.commands.setContent(draft, { emitUpdate: false });
      schedule();
    }
  };

  $effect(() => {
    void note;
    void steamId;
    if (editor) untrack(sync);
  });

  $effect(() => {
    editor?.view.dom.setAttribute("aria-label", t.account_notes());
  });

  const send = async (target: string, html: string): Promise<void> => {
    try {
      const result = await run("set_account_note", { steamId: target, note: html });
      if (result) {
        if (drafts.get(target) === html) drafts.delete(target);
        if (loaded.steamId === target) loaded = { steamId: target, note: html, html };
      } else {
        drafts.set(target, html);
        if (loaded.steamId === target) {
          failed = html;
          if (queued === html) queued = null;
        }
      }
    } finally {
      pending -= 1;
      if (pending === 0) sync();
    }
  };

  const save = (): void => {
    if (timer) clearTimeout(timer);
    timer = null;
    const target = loaded.steamId;
    if (!target || !editor || editor.isDestroyed) return;
    const html = read();
    tooLong = noteBytes(html) > MAX_NOTE_BYTES;
    if (html === (queued ?? loaded.html)) {
      if (pending === 0) drafts.delete(target);
      return;
    }
    if (tooLong) {
      drafts.set(target, html);
      return;
    }
    if (html === failed) return;
    queued = html;
    pending += 1;
    queue = queue.then(() => send(target, html));
  };

  const schedule = (): void => {
    if (timer) clearTimeout(timer);
    timer = setTimeout(save, SAVE_DELAY_MS);
  };

  const blur = (): void => {
    focused = false;
    save();
    sync();
  };

  const active = $derived.by(() => {
    void revision;
    return Object.fromEntries(
      FORMATS.map(({ name }) => [name, focused && Boolean(editor?.isActive(name))]),
    );
  });

  const activeColor = $derived.by(() => {
    void revision;
    if (!focused) return null;
    const color: unknown = editor?.getAttributes("textStyle").color;
    const hex = typeof color === "string" ? toHex(color) : null;
    return hex && NOTE_COLORS.includes(hex) ? hex : null;
  });

  const MIN_HEIGHT = 74;

  // ! A grip of our own rather than CSS `resize`, which WebKitGTK does not offer on this editor.
  const startResize = (event: PointerEvent): void => {
    if (!host || event.button !== 0) return;
    event.preventDefault();
    const grip = event.currentTarget as HTMLElement;
    const startY = event.clientY;
    const startHeight = host.getBoundingClientRect().height;
    grip.setPointerCapture(event.pointerId);
    const move = (next: PointerEvent): void => {
      height = Math.max(MIN_HEIGHT, Math.round(startHeight + next.clientY - startY));
    };
    const stop = (): void => {
      grip.removeEventListener("pointermove", move);
      grip.removeEventListener("pointerup", stop);
      grip.removeEventListener("pointercancel", stop);
    };
    grip.addEventListener("pointermove", move);
    grip.addEventListener("pointerup", stop);
    grip.addEventListener("pointercancel", stop);
  };

  const format = (apply: (chain: ChainedCommands) => ChainedCommands): void => {
    if (editor) apply(editor.chain().focus()).run();
  };

  onMount(() => {
    if (!host) return;
    editor = new Editor({
      element: host,
      extensions: [
        StarterKit.configure({
          paragraph: false,
          heading: false,
          blockquote: false,
          codeBlock: false,
          code: false,
          horizontalRule: false,
          link: false,
        }),
        // ! Stored notes write their paragraphs as `div`.
        Paragraph.extend({
          parseHTML: () => [{ tag: "p" }, { tag: "div" }],
        }),
        TextStyle,
        Color,
        Placeholder.configure({ placeholder: () => t.account_notes_placeholder() }),
      ],
      editorProps: {
        attributes: {
          class: "notes-prose",
          role: "textbox",
          "aria-multiline": "true",
          spellcheck: "true",
        },
      },
      onUpdate: schedule,
      // ! ProseMirror reports focus changes from inside DOM updates, which may be Svelte's own, so state is only written once that update is over.
      onFocus: () => {
        queueMicrotask(() => {
          focused = true;
        });
      },
      onBlur: () => {
        queueMicrotask(blur);
      },
      onTransaction: () => {
        queueMicrotask(() => {
          revision += 1;
        });
      },
    });
  });

  onDestroy(() => {
    save();
    editor?.destroy();
  });
</script>

<div class="notes">
  <div class="notes-toolbar" role="toolbar" aria-label={t.account_notes()}>
    {#each FORMATS as { name, icon: Icon, label, apply } (name)}
      <button
        class="notes-tool"
        class:active={active[name]}
        type="button"
        title={label()}
        aria-label={label()}
        aria-pressed={active[name]}
        onmousedown={(event) => event.preventDefault()}
        onclick={() => format(apply)}
      >
        <Icon size={15} />
      </button>
    {/each}
    <span class="notes-separator"></span>
    {#each NOTE_COLORS as color (color)}
      <button
        class="notes-color"
        class:active={activeColor === color}
        type="button"
        title={t.notes_colour()}
        aria-label={`${t.notes_colour()} ${color}`}
        aria-pressed={activeColor === color}
        style={`background:${color}`}
        onmousedown={(event) => event.preventDefault()}
        onclick={() =>
          format((chain) => (activeColor === color ? chain.unsetColor() : chain.setColor(color)))}
      ></button>
    {/each}
    <span class="notes-separator"></span>
    <button
      class="notes-tool"
      type="button"
      title={t.notes_clear_format()}
      aria-label={t.notes_clear_format()}
      onmousedown={(event) => event.preventDefault()}
      onclick={() => format((chain) => chain.unsetAllMarks())}
    >
      <RemoveFormatting size={15} />
    </button>
  </div>
  <div class="notes-frame">
    <div
      class="notes-editor input"
      class:invalid={tooLong}
      style:height={height === null ? null : `${height}px`}
      bind:this={host}
    ></div>
    <span
      class="notes-grip"
      aria-hidden="true"
      onpointerdown={startResize}
      ondblclick={() => (height = null)}
    ></span>
  </div>
  {#if tooLong}
    <p class="notes-error" role="alert">{t.error_note_too_long()}</p>
  {/if}
</div>
