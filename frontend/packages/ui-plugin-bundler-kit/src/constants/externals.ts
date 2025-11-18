const GLOBALS = {
  vue: "Vue",
  "vue-router": "VueRouter",
  pinia: "Pinia",
  "@vueuse/core": "VueUse",
  "@vueuse/components": "VueUse",
  "@vueuse/router": "VueUse",
  "@flow-dev/console-shared": "HaloConsoleShared",
  "@flow-dev/components": "HaloComponents",
  "@flow-dev/api-client": "HaloApiClient",
  "@flow-dev/richtext-editor": "RichTextEditor",
  axios: "axios",
};

const EXTERNALS = Object.keys(GLOBALS) as string[];

export { EXTERNALS, GLOBALS };
