export type MessagePartType = {
  text: string;
  bold?: boolean;
  code?: boolean;
};

export type MessageType = string | MessagePartType[];

export const messageParts = (content: MessageType): MessagePartType[] => {
  const parts = typeof content === "string" ? [{ text: content }] : content;
  return parts.flatMap((part) => {
    if (part.bold || part.code) return [part];
    return part.text
      .split(/(\.maFile\b|\bmanifest\.json\b|\bconfig\/loginusers\.vdf\b)/g)
      .filter(Boolean)
      .map((text) =>
        /^(\.maFile|manifest\.json|config\/loginusers\.vdf)$/.test(text)
          ? { text, code: true }
          : { text },
      );
  });
};

export const containsAccountName = (content: MessageType, name: string): boolean => {
  return Array.isArray(content) && content.some((part) => part.bold && part.text === name);
};
