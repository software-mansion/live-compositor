let nextStreamNumber = 1;

/*
 * Generates unique input stream id that can be used in e.g. Mp4 component
 */
export function newInternalStreamId(): number {
  const result = nextStreamNumber;
  nextStreamNumber += 1;
  return result;
}
