import type { _smelterInternals } from '@swmansion/smelter';

export type Logger = _smelterInternals.Logger;

export enum LoggerLevel {
  ERROR = 'error',
  WARN = 'warn',
  INFO = 'info',
  DEBUG = 'debug',
  TRACE = 'trace',
}
