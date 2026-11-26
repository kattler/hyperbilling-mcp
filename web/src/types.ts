export type Locale = "en" | "ko";

export type PageName = "guide" | "api";

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

export interface View {
  name: string;
  path: string;
}

export interface DocsData {
  page: PageName;
  locale: Locale;
  version: string;
  built: string;
  tools: Tool[];
  views: View[];
}
