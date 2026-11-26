export type Locale = "en" | "ko";

export interface Argument {
  name: string;
  kind: string;
  description: string;
  required: boolean;
  values: string[];
}

export interface Tool {
  name: string;
  description: string;
  method: string;
  path: string;
  arguments: Argument[];
}

export interface DocsData {
  locale: Locale;
  version: string;
  built: string;
  tools: Tool[];
}
