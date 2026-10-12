import { defineConfig } from "oxfmt";

export default defineConfig({
   printWidth: 100,
   semi: true,
   tabWidth: 3,
   trailingComma: "all",

   sortImports: {
      internalPattern: ["~"],
      newlinesBetween: true,
      groups: [
         "side_effect",
         "builtin",
         "external",
         "internal",
         { newlinesBetween: false },
         ["parent", "sibling", "index"],
         ["side_effect_style", "style"],
         "unknown",
      ],
   },

   sortPackageJson: {
      sortScripts: true,
   },

   sortTailwindcss: {
      stylesheet: "src/styles.css",
      functions: ["cn"],
   },

   overrides: [
      {
         files: ["*.yaml", "*.yml"],
         options: {
            tabWidth: 2,
         },
      },
   ],

   ignorePatterns: ["migrations/", "**/*.md", "**/*.gen.ts"],
});
