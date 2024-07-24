import { Context } from "./types";
import { state } from './hookState';

//export function useWithInterval(intervalMs: number): number {
//  console.log('TODO: useWithInterval');
//  return 0;
//}
//
//type UpdateFunction = () => void;
//type CleanupFunction = () => void;
//type UseExternalHook = (triggerUpdate: UpdateFunction) => CleanupFunction;
//
//export function useExternal(hookFn: UseExternalHook, dependencies: any[] = []): number {
//  console.log('TODO: useWithInterval');
//  return 0;
//}

export function useContext(): Context {
  return state().useContext();
}
