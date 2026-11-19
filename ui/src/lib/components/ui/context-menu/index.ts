import { ContextMenu as ContextMenuPrimitive } from "bits-ui";
import Content from "./context-menu-content.svelte";
import Item from "./context-menu-item.svelte";
import RadioItem from "./context-menu-radio-item.svelte";
import Separator from "./context-menu-separator.svelte";
import SubContent from "./context-menu-sub-content.svelte";
import SubTrigger from "./context-menu-sub-trigger.svelte";

const Root = ContextMenuPrimitive.Root;
const Trigger = ContextMenuPrimitive.Trigger;
const Sub = ContextMenuPrimitive.Sub;
const RadioGroup = ContextMenuPrimitive.RadioGroup;

export {
  Root,
  Trigger,
  Sub,
  Content,
  Item,
  RadioGroup,
  RadioItem,
  Separator,
  SubContent,
  SubTrigger,
  Root as ContextMenu,
  Trigger as ContextMenuTrigger,
  Sub as ContextMenuSub,
  Content as ContextMenuContent,
  Item as ContextMenuItem,
  RadioGroup as ContextMenuRadioGroup,
  RadioItem as ContextMenuRadioItem,
  Separator as ContextMenuSeparator,
  SubContent as ContextMenuSubContent,
  SubTrigger as ContextMenuSubTrigger,
};
