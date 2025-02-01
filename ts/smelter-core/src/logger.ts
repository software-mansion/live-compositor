import type { _liveCompositorInternals } from 'live-compositor';

export type Logger = _liveCompositorInternals.Logger;

export enum LoggerLevel {
  ERROR = 'error',
  WARN = 'warn',
  INFO = 'info',
  DEBUG = 'debug',
  TRACE = 'trace',
}
