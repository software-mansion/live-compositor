import type { Action, ThunkAction } from "@reduxjs/toolkit"
import { combineSlices, configureStore, createSlice } from "@reduxjs/toolkit"

export const showInstructionsSlice = createSlice({
  name: "instructions",
  initialState: true,
  reducers: create => ({
    toggle: create.reducer(state => !state)
  }),
  selectors: {
    shouldShow: state => state,
  }
})

// `combineSlices` automatically combines the reducers using
// their `reducerPath`s, therefore we no longer need to call `combineReducers`.
const rootReducer = combineSlices(showInstructionsSlice)
// Infer the `RootState` type from the root reducer
export type RootState = ReturnType<typeof rootReducer>

export const makeStore = (preloadedState?: Partial<RootState>) => {
  const store = configureStore({
    reducer: rootReducer,
    preloadedState,
  })
  return store
}

export const store = makeStore()

// Infer the type of `store`
export type AppStore = typeof store
// Infer the `AppDispatch` type from the store itself
export type AppDispatch = AppStore["dispatch"]
export type AppThunk<ThunkReturnType = void> = ThunkAction<
  ThunkReturnType,
  RootState,
  unknown,
  Action
>
