// ai-generated
export type LoadError = "invalid_url" | "unreachable" | "load_failed";

export type Tab = {
  id: string;
  title: string;
  url: string;
  favicon_url: string;
  engine: "native" | "toy";
  load_error: LoadError | null;
};

export type SessionSnapshot = {
  tabs: Tab[];
  active_tab_id: string;
  current_url: string;
  can_go_back: boolean;
  can_go_forward: boolean;
  is_bookmarked: boolean;
  load_error: LoadError | null;
  persist_mode: "disk" | "memory";
};

export type PersistNotice = {
  kind: "error" | "warning";
  message: string;
};

export type BookmarkRow = {
  url: string;
  title: string;
  created_at: number;
};

export type VisitRow = {
  id: number;
  url: string;
  title: string;
  visited_at: number;
};

export type ChromeHandlers = {
  onNavigate: (tabId: string, input: string) => void;
  onNewTab: () => void;
  onCloseTab: (tabId: string) => void;
  onSwitchTab: (tabId: string) => void;
  onBack: (tabId: string) => void;
  onForward: (tabId: string) => void;
  onReload: (tabId: string) => void;
  onToggleBookmark: (tabId: string) => void;
  onOpenLibrary: () => void;
};
// ai-generated end
