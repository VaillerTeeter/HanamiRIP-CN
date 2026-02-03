export interface MonthAnime {
  id: number;
  name: string;
  nameCn?: string;
  alias?: string;
  origin?: string;
  airedCount?: number;
  totalCount?: number;
  summaryCn?: string;
  summaryTranslated?: boolean;
  summaryTranslateFailed?: boolean;
  types?: string[];
  regions?: string[];
  audiences?: string[];
  image: string;
  date?: string;
  rating?: number | null;
  summary?: string;
  url?: string;
  month?: number;
}

export interface StaffPerson {
  id: number;
  name: string;
  url: string;
}

export interface StaffGroup {
  role: string;
  people: StaffPerson[];
}

export interface CharacterLink {
  id: number;
  name: string;
  url: string;
  relation?: string;
}

export interface SeasonMonthData {
  year: number;
  month: number;
  count: number;
  list: MonthAnime[];
}

export interface SeasonResponse {
  year: number;
  season: string;
  fetchedAt: string;
  source: string;
  months: SeasonMonthData[];
}
