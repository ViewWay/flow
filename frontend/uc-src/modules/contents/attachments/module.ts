import { definePlugin } from "@flow-dev/console-shared";
import { defineAsyncComponent } from "vue";

export default definePlugin({
  components: {
    AttachmentSelectorModal: defineAsyncComponent(
      () => import("./components/AttachmentSelectorModal.vue")
    ),
  },
});
