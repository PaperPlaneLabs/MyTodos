export interface ContinueWithoutTimerActions {
  startTimer(taskId: number): Promise<void>;
  notifyTimerContinued(): Promise<void>;
  closeWindow(): Promise<void>;
}

export interface SwitchTaskActions {
  notifySwitchRequested(): Promise<void>;
  focusMainWindow(): Promise<void>;
  closeWindow(): Promise<void>;
}

export async function continueWithoutTimer(
  taskId: number,
  actions: ContinueWithoutTimerActions,
): Promise<void> {
  await actions.startTimer(taskId);
  await actions.notifyTimerContinued();
  await actions.closeWindow();
}

export async function switchTask(actions: SwitchTaskActions): Promise<void> {
  await actions.notifySwitchRequested();
  await actions.focusMainWindow();
  await actions.closeWindow();
}
