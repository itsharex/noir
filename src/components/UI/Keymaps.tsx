import { For } from 'solid-js';

// TODO: use t instead
const ALL_KEYMAPS = [
  { action: 'Help', keys: ['F1'] },
  { action: 'Execute query', keys: ['Ctrl', 'E/Enter'] },
  { action: 'Select tab', keys: ['Alt/Cmd', 'number'] },
  { action: 'Select connection tab', keys: ['Ctrl/Cmd', 'number'] },
  { action: 'New tab', keys: ['Ctrl/Cmd', 'T'] },
  { action: 'Close current tab', keys: ['Ctrl/Cmd', 'W'] },
  { action: 'Focus on editor', keys: ['Ctrl', 'L'] },
  { action: 'Format query', keys: ['Ctrl', 'Shift', 'F'] },
  { action: 'Trigger autocomplete in editor', keys: ['Ctrl', 'Space'] },
  { action: 'Select next/previous result', keys: ['Ctrl', 'Shift', 'N/P'] },
  { action: 'Select next/previous page', keys: ['Ctrl', 'N/P'] },
  { action: 'Add row', keys: ['Alt/Cmd', 'N'] },
];

const LIMIT = 10;

const Keymaps = (props: { short?: boolean }) => {
  const keymaps = props.short ? ALL_KEYMAPS : ALL_KEYMAPS.slice(0, LIMIT);
  return (
    <div class="grid grid-cols-2">
      <div class="flex flex-col items-end pr-1">
        <For each={keymaps}>
          {({ action }) => (
            <div class="h-[40px] flex items-center align-middle text-base-content">
              <span class="text-lg font-medium">{action}</span>
            </div>
          )}
        </For>
      </div>
      <div class="flex flex-col items-start pl-1">
        <For each={keymaps}>
          {({ keys }) => (
            <div class="h-[40px] flex items-center">
              <span class="flex gap-2">
                <For each={keys}>{(key) => <kbd class="kbd kbd-sm text-base-content">{key}</kbd>}</For>
              </span>
            </div>
          )}
        </For>
      </div>
    </div>
  );
};

export default Keymaps;
