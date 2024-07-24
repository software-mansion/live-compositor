import { RenderContext } from './context';
import { Context } from './types';

type State = {
  useContext: () => Context;
};

const hooksState: { state: State | null } = { state: null };

export function state(): State {
  if (!hooksState.state) {
    throw new Error('Hook context not populated.');
  }
  return hooksState.state;
}

export function setupHookState(context: RenderContext) {
  if (hooksState.state) {
    console.error('Error setting up hook context, state already populated.');
  }
  hooksState.state = {
    useContext: () => context.publicContext,
  };
}

export function removeHookState() {
  hooksState.state = null;
}
