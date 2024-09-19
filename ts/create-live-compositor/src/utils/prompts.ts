import { constants } from 'os';
import prompts, { Answers, Options, Choice as PromptChoice, PromptObject } from 'prompts';

export interface Choice<T> extends PromptChoice {
  value: T;
}

async function promptWrapper<T extends string = string>(
  questions: PromptObject<T> | Array<PromptObject<T>>,
  options?: Options
): Promise<Answers<T>> {
  return await prompts<T>(questions, {
    onCancel() {
      process.exit(constants.signals.SIGINT + 128); // Exit code 130 used when process is interrupted with ctrl+c.
    },
    ...options,
  });
}

export async function confirmPrompt(message: string, initial?: boolean): Promise<boolean> {
  const { value } = await promptWrapper({
    type: 'confirm',
    message,
    name: 'value',
    initial: initial ?? false,
  });
  return !!value;
}

export async function selectPrompt<T>(message: string, choices: Choice<T>[]): Promise<T> {
  const { value } = await promptWrapper({ type: 'select', message, name: 'value', choices });
  return value;
}

export async function textPrompt(message: string, initial?: string): Promise<string> {
  const { value } = await promptWrapper({ type: 'text', message, name: 'value', initial });
  return value;
}
