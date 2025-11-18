import BasicLayout from "@console/layouts/BasicLayout.vue";
import { IconTerminalBoxLine } from "@flow-dev/components";
import { definePlugin } from "@flow-dev/console-shared";
import { markRaw } from "vue";
import Overview from "./Overview.vue";

export default definePlugin({
  components: {},
  routes: [
    {
      path: "/overview",
      name: "OverviewRoot",
      component: BasicLayout,
      meta: {
        title: "core.overview.title",
        searchable: true,
        permissions: ["system:actuator:manage"],
        menu: {
          name: "core.sidebar.menu.items.overview",
          group: "system",
          icon: markRaw(IconTerminalBoxLine),
          priority: 3,
        },
      },
      children: [
        {
          path: "",
          name: "Overview",
          component: Overview,
        },
      ],
    },
  ],
});
