import { For } from "solid-js";
import { useAppSelector } from "../../services/Context";
import { Alert } from "./Alert";

export const Alerts = () => {
  const {
    messages: { messages },
  } = useAppSelector();
  return (
    <div class="absolute">
      <div class="toast whitespace-normal min-w-[300px] w-[600px] z-50">
        <For each={messages}>
          {(msg) => (
            <Alert color={msg.type}>
              <span class="font-medium">{msg.message}</span>
            </Alert>
          )}
        </For>
      </div>
    </div>
  );
};
