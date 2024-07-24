import { Context } from './types';

export type RegisterEvent = RegisterInterval | RegisterPts;

export type RegisterInterval = {
  type: 'interval';
  intervalMs: number;
};

export function isRegisterInterval(event: RegisterEvent): event is RegisterInterval {
  return event.type === 'interval';
}

export type RegisterPts = {
  type: 'pts';
  pts: number;
};

export function isRegisterPts(event: RegisterEvent): event is RegisterPts {
  return event.type === 'pts';
}

export type RenderContext = {
  publicContext: Context;
  registerCallback: (event: RegisterEvent) => void;
};
