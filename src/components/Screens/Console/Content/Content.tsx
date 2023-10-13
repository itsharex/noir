import { For, Match, Show, Switch } from "solid-js";
import { useAppSelector } from "services/Context";
import { AddIcon, CloseIcon } from "components/UI/Icons";
import { t } from "utils/i18n";
import { QueryTab } from "./QueryTab/QueryTab";
import { TableStructureTab } from "./TableStructure/TableStructureTab";
import { ContentTab } from "services/Connections";

export const Content = () => {
  const {
    connections: {
      contentStore,
      setContentIdx,
      addContentTab,
      removeContentTab,
    },
  } = useAppSelector();

  return (
    <div class="flex flex-col h-full">
      <div class="bg-base-300 tabs gap-1">
        <For each={contentStore.tabs}>
          {(_tab, idx) => (
            <div
              class="tab tab-md tab-lifted !pr-1 !pl-1"
              classList={{ "tab-active": contentStore.idx === idx() }}
            >
              <button
                class="text-xs p-1 px-3"
                tabindex={0}
                onClick={() => setContentIdx(idx())}
              >
                {t('console.query')} #{idx() + 1}
              </button>
              <Show when={idx() > 0}>
                <button
                  onClick={() => removeContentTab(idx())}
                  class="ml-2 mb-1 p-1"
                >
                  <CloseIcon />
                </button>
              </Show>
            </div>
          )}
        </For>
        <div class="tab tab-sm tab-lifted tab-active">
          <button onClick={() => addContentTab()}>
            <AddIcon />
          </button>
        </div>
      </div>
      <div class="flex-1">
        <For each={contentStore.tabs}>
          {(tab, idx) => (
            <Show when={contentStore.idx === idx()}>
              <Switch>
                <Match when={tab.key === ContentTab.Query}>
                  <QueryTab />
                </Match>
                <Match when={tab.key === ContentTab.TableStructure}>
                  <TableStructureTab />
                </Match>
              </Switch>
            </Show>
          )}
        </For>
      </div>
    </div>
  );
};
