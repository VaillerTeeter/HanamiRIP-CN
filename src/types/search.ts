export type LogicOp = "and" | "or" | "not";

export type SearchTerm = { value: string; op: LogicOp; source: "preset" | "custom" | "tracked" };

export type SearchResult = {
  title: string;
  detailUrl?: string;
  magnet?: string;
  download?: string;
  size?: string;
  date?: string;
};
