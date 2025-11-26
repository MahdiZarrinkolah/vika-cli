import {
  Breadcrumb,
  BreadcrumbHome,
  Breadcrumbs,
  BreadcrumbSeparator,
} from "@/components/breadcrumbs";
import { ContentLink } from "@/components/content-link";
import { Logo } from "@/components/logo";
import { PageSection } from "@/components/page-section";
import { SidebarLayoutContent } from "@/components/sidebar-layout";
import { getModules, type Module } from "@/data/lessons";
import { BookIcon } from "@/icons/book-icon";
import { LessonsIcon } from "@/icons/lessons-icon";
import type { Metadata } from "next";
import Link from "next/link";

export const metadata: Metadata = {
  title: "Vika Documentation",
  description:
    "Complete documentation for Vika - Generate TypeScript code from OpenAPI/Swagger specifications.",
};

export default async function Page() {
  let modules = await getModules();
  let lessons = modules.flatMap(({ lessons }) => lessons);

  return (
    <SidebarLayoutContent
      breadcrumbs={
        <Breadcrumbs>
          <BreadcrumbHome />
          <BreadcrumbSeparator />
          <Breadcrumb>Overview</Breadcrumb>
        </Breadcrumbs>
      }
    >
      <div className="relative mx-auto max-w-7xl">
        <div className="absolute -inset-x-2 top-0 -z-10 h-80 overflow-hidden rounded-t-2xl mask-b-from-60% sm:h-88 md:h-112 lg:-inset-x-4 lg:h-128">
          <img
            alt=""
            src="https://assets.tailwindcss.com/templates/compass/hero-background.png"
            className="absolute inset-0 h-full w-full mask-l-from-60% object-cover object-center opacity-40"
          />
          <div className="absolute inset-0 rounded-t-2xl outline-1 -outline-offset-1 outline-gray-950/10 dark:outline-white/10" />
        </div>
        <div className="mx-auto max-w-6xl">
          <div className="relative">
            <div className="px-4 pt-48 pb-12 lg:py-24">
              <Logo className="h-8 fill-gray-950 dark:fill-white" />
              <h1 className="sr-only">Course overview</h1>
              <p className="mt-7 max-w-lg text-base/7 text-pretty text-gray-600 dark:text-gray-400">
                Complete documentation for Vika - Generate TypeScript code from OpenAPI/Swagger specifications.
              </p>
              <div className="mt-6 flex flex-wrap items-center gap-x-4 gap-y-3 text-sm/7 font-semibold text-gray-950 sm:gap-3 dark:text-white">
                <div className="flex items-center gap-1.5">
                  <BookIcon className="stroke-gray-950/40 dark:stroke-white/40" />
                  {modules.length} sections
                </div>
                <span className="hidden text-gray-950/25 sm:inline dark:text-white/25">
                  &middot;
                </span>
                <div className="flex items-center gap-1.5">
                  <LessonsIcon className="stroke-gray-950/40 dark:stroke-white/40" />
                  {lessons.length} topics
                </div>
              </div>
              <div className="mt-10">
                <Link
                  href={`/${modules[0].lessons[0].id}`}
                  className="inline-flex items-center gap-x-2 rounded-full bg-gray-950 px-3 py-0.5 text-sm/7 font-semibold text-white hover:bg-gray-800 dark:bg-gray-700 dark:hover:bg-gray-600"
                >
                  Get Started
                </Link>
              </div>
            </div>

            <div className="grid grid-cols-1 gap-y-16 pb-10 sm:px-4">
              {modules.map((module: Module, index: number) => (
                <PageSection
                  key={module.id}
                  id={module.id}
                  title={`Part ${index + 1}`}
                >
                  <div className="max-w-2xl">
                  <h2 className="text-2xl/7 font-medium tracking-tight text-pretty text-gray-950 dark:text-white">
                    {module.title}
                  </h2>
                  <p className="mt-4 text-base/7 text-gray-700 sm:text-sm/7 dark:text-gray-400">
                    {module.description}
                  </p>

                  <ol className="mt-6 space-y-4">
                    {module.lessons.map((lesson) => (
                      <li key={lesson.id}>
                        <ContentLink
                          title={lesson.title}
                          description={lesson.description}
                          href={`/${lesson.id}`}
                          type="article"
                        />
                      </li>
                    ))}
                  </ol>
                  </div>
                </PageSection>
              ))}
            </div>
          </div>
        </div>
      </div>
    </SidebarLayoutContent>
  );
}
