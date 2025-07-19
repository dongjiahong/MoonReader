import { createRouter, createWebHistory } from "vue-router";
import HomeView from "../views/HomeView.vue";

const routes = [
  {
    path: "/",
    name: "home",
    component: HomeView,
  },
  {
    path: "/knowledge-bases",
    name: "knowledge-bases",
    component: () =>
      import(
        /* webpackChunkName: "knowledge-bases" */ "../views/KnowledgeBasesView.vue"
      ),
  },
  {
    path: "/knowledge-bases/:id",
    name: "knowledge-base-detail",
    component: () =>
      import(
        /* webpackChunkName: "knowledge-base-detail" */ "../views/KnowledgeBaseDetailView.vue"
      ),
  },
  {
    path: "/knowledge-bases/:id/documents",
    name: "documents",
    component: () =>
      import(/* webpackChunkName: "documents" */ "../views/DocumentsView.vue"),
  },
  {
    path: "/knowledge-bases/:id/quiz",
    name: "ai-quiz",
    component: () =>
      import(/* webpackChunkName: "ai-quiz" */ "../views/AIQuizView.vue"),
  },
  {
    path: "/knowledge-bases/:id/review",
    name: "review",
    component: () =>
      import(/* webpackChunkName: "review" */ "../views/ReviewView.vue"),
  },
  {
    path: "/knowledge-bases/:id/history",
    name: "history",
    component: () =>
      import(/* webpackChunkName: "history" */ "../views/HistoryView.vue"),
  },
  {
    path: "/settings",
    name: "settings",
    component: () =>
      import(/* webpackChunkName: "settings" */ "../views/SettingsView.vue"),
  },
];

const router = createRouter({
  history: createWebHistory(process.env.BASE_URL),
  routes,
});

export default router;
