import { SidebarLayout } from "@/components/sidebar-layout";
import { getModules } from "@/data/docs";
import type React from "react";

export default function DocsLayout({
  children,
}: {
  children: React.ReactNode;
}) {
  return <SidebarLayout modules={getModules()}>{children}</SidebarLayout>;
}
