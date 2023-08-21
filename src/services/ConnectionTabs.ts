import { createStore } from "solid-js/store";
import { ConnectionConfig, DbSchema } from "../interfaces";
import { onMount } from "solid-js";
import { Store } from "tauri-plugin-store-api";
import { debounce } from "utils/utils";
import { invoke } from "@tauri-apps/api";

const store = new Store(".connections.dat");

export const MessageType = {
  error: "error",
  info: "info",
  success: "success",
  warning: "warning",
} as const;

export const ContentComponent = {
  QueryTab: "QueryTab",
  TableStructureTab: "TableStructureTab",
} as const;

type ContentComponentKeys = keyof typeof ContentComponent;

export const newContentTab = (label: string, key: ContentComponentKeys) => {
  switch (key) {
    case ContentComponent.QueryTab:
      return {
        label,
        data: { query: "", results: [] },
        key,
      };
    case ContentComponent.TableStructureTab:
      return {
        label,
        data: {
          table: "",
          structure: [],
          indices: [],
          foreignKeys: [],
          triggers: [],
        },
        key,
      };
    default:
      throw new Error("Invalid content tab key");
  }
};

export type QueryContentTabData = {
  query: string;
  results: Record<string, any>[];
};

export type TableStructureContentTabData = {
  table: string;
  structure: Record<string, any>[];
  indices: Record<string, any>[];
  foreignKeys: Record<string, any>[];
  triggers: Record<string, any>[];
};

export type ContentTab = {
  label: string;
  error?: {
    message: string;
    type: keyof typeof MessageType;
  };
} & (
    | {
      key: typeof ContentComponent.QueryTab;
      data: QueryContentTabData;
    }
    | {
      key: typeof ContentComponent.TableStructureTab;
      data: TableStructureContentTabData;
    }
  );

type ConnectionTab = {
  label: string;
  id: string;
  schema: DbSchema;
  connection: ConnectionConfig;
};

const CONNECTIONS_TABS_KEY = "_conn_tabs";
const CONTENT_TABS_KEY = "_content_tabs";

const getSavedData = async (key: string, defaultValue: any = []) => {
  const str = await store.get(key);
  if (!str) return defaultValue;
  try {
    const res = JSON.parse(str as string);
    return res as unknown;
  } catch (e) {
    return defaultValue;
  }
};

type ConnectionStore = {
  tabs: ConnectionTab[];
  idx: number;
};

type ContentStore = {
  tabs: ContentTab[];
  idx: number;
};

export const ConnectionTabsService = () => {
  const [connectionStore, setConnectionStore] = createStore<ConnectionStore>({
    tabs: [],
    idx: 0,
  });

  const [contentStore, setContentStore] = createStore<ContentStore>({
    tabs: [],
    idx: 0,
  });

  onMount(async () => {
    const conn_tabs: ConnectionStore = await getSavedData(CONNECTIONS_TABS_KEY);
    const tabs = await conn_tabs.tabs.reduce(async (acc, conn) => {
      const res = await acc;
      try {
        await invoke("init_connection", { config: conn.connection });
        return Promise.resolve([...res, conn]);
      } catch (e) {
        console.log(e);
        return acc;
      }
    }, Promise.resolve([] as ConnectionTab[]));
    setConnectionStore(() => ({ ...conn_tabs, tabs }));
    const content = await getSavedData(CONTENT_TABS_KEY);
    setContentStore(() => content as ContentStore);
  });

  const updateStore = debounce(async () => {
    await store.set(CONNECTIONS_TABS_KEY, JSON.stringify(connectionStore));
    await store.set(CONTENT_TABS_KEY, JSON.stringify(contentStore));
    await store.save();
  }, 1000);

  const addTab = async (tab: ConnectionTab) => {
    if (connectionStore.tabs.find((t) => t.id === tab.id)) return;
    setConnectionStore("tabs", connectionStore.tabs.concat(tab));
    setContentStore("tabs", [newContentTab("Query", ContentComponent.QueryTab)]);
    const idx = connectionStore.tabs.length;
    setConnectionStore("idx", idx);
    updateStore();
  };

  const removeTab = async (id: string) => {
    setConnectionStore(
      "tabs",
      connectionStore.tabs.filter((t) => t.id !== id)
    );
    setConnectionStore("idx", 0);
    updateStore();
  };

  const clearStore = async () => {
    await store.clear();
  };

  const getActiveConnection = () => {
    return connectionStore.tabs[connectionStore.idx - 1];
  };

  const getActiveContentTab = () => {
    return contentStore.tabs[contentStore.idx];
  };

  const setActiveContentQueryTabData = (data: QueryContentTabData) => {
    const tab = getActiveContentTab();
    if (!tab) return;
    setContentStore(
      "tabs",
      contentStore.tabs.map((t, i) =>
        i === contentStore.idx ? { ...t, data, key: ContentComponent.QueryTab } : t
      )
    );
  };

  const resetActiveContentQueryTabMessage = () => {
    const tab = getActiveContentTab();
    if (!tab) return;
    setContentStore(
      "tabs",
      contentStore.tabs.map((t, i) =>
        i === contentStore.idx ? { ...t, error: undefined } : t
      )
    );
  };

  const setActiveContentQueryTabMessage = (
    type: keyof typeof MessageType,
    message: string
  ) => {
    const tab = getActiveContentTab();
    if (!tab) return;
    setContentStore(
      "tabs",
      contentStore.tabs.map((t, i) =>
        i === contentStore.idx ? { ...t, error: { type, message } } : t
      )
    );
  };

  return {
    connectionStore,
    setConnectionStore,
    contentStore,
    setContentStore,
    addTab,
    removeTab,
    clearStore,
    getActiveConnection,
    getActiveContentTab,
    setActiveContentQueryTabData,
    setActiveContentQueryTabMessage,
    resetActiveContentQueryTabMessage,
    updateStore,
  };
};
