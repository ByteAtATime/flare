import * as protocol from "../protocol";

export type Content =
  | {
      text: string;
    }
  | {
      file: string;
    }
  | {
      html: string;
      text?: string;
    };

export type ReadContent = {
  text: string;
  file?: string;
  html?: string;
};

export type CopyOptions = {
  concealed?: boolean;
};

const Clipboard = {
  async copy(
    content: string | number | Content,
    options?: CopyOptions
  ): Promise<void> {
    let normalized: protocol.ClipboardContent;
    if (typeof content === "string") {
      normalized = { text: content };
    } else if (typeof content === "number") {
      normalized = { text: String(content) };
    } else {
      normalized = content;
    }
    await protocol.clipboardCopy(normalized, options?.concealed ?? false);
  },

  async paste(content: string | Content): Promise<void> {
    console.warn("not implemented");
  },

  async clear(): Promise<void> {
    await protocol.clipboardClear();
  },

  async read(options?: { offset?: number }): Promise<ReadContent> {
    return await protocol.clipboardRead(options?.offset);
  },

  async readText(options?: { offset?: number }): Promise<string | undefined> {
    const { text } = await protocol.clipboardRead(options?.offset);
    return text || undefined;
  },
};

export { Clipboard };
