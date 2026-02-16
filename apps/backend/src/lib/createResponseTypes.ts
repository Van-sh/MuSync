import { StatusMap } from "elysia";
import type { AnySchema, Prettify } from "elysia/types";

type ValidResponseMap = { [K in keyof StatusMap]?: AnySchema };

export type ResponseMapOutput<T extends ValidResponseMap> = {
   [K in keyof T as K extends keyof StatusMap ? StatusMap[K] : never]: T[K];
};

export function createResponseTypes<T extends ValidResponseMap>(
   map: T,
): Prettify<ResponseMapOutput<T>> {
   const result = {} as ResponseMapOutput<T>;
   for (const key of Object.keys(map)) {
      const code = StatusMap[key as keyof StatusMap];
      if (code !== undefined) {
         (result as Record<number, AnySchema>)[code] = map[key as keyof T] as AnySchema;
      }
   }
   return result;
}
