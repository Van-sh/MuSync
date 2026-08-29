import { type Prettify } from "~/types";

export type Store<State, Actions> = Prettify<State & { actions: Prettify<Actions> }>;
