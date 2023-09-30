import { actions } from "./actions";
import { useAppSelector } from "services/Context";
import { CommandPalette, Root } from "solid-command-palette";
import { JSX } from "solid-js/jsx-runtime";
import { createShortcut } from "@solid-primitives/keyboard";

export interface ActionsContext {
  [key: string]: any;
}

export const CommandPaletteContext = (props: { children: JSX.Element }) => {
  const {
    connectionsService: {
      addContentTab,
      removeContentTab,
      setContentIdx,
      setConnectionIdx,
    },
  } = useAppSelector();

  const actionsContext: ActionsContext = {};

  for (let i = 0; i < 9; i++) {
    createShortcut(["Alt", String(i)], () => setContentIdx(i - 1));
    createShortcut(["Control", String(i)], () => setConnectionIdx(i - 1));
  }

  createShortcut(["Control", "T"], () => addContentTab());
  createShortcut(["Control", "Shift", "T"], () => removeContentTab());

  return (
    <Root actions={actions} actionsContext={actionsContext}>
      <CommandPalette />
      {props.children}
    </Root>
  );
};
