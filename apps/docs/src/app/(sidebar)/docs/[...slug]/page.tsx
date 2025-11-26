import {
  Breadcrumb,
  BreadcrumbHome,
  Breadcrumbs,
  BreadcrumbSeparator,
} from "@/components/breadcrumbs";
import { NextPageLink } from "@/components/next-page-link";
import { SidebarLayoutContent } from "@/components/sidebar-layout";
import TableOfContents from "@/components/table-of-contents";
import { Video } from "@/components/video-player";
import { getLesson, getLessonContent } from "@/data/docs";
import type { Metadata } from "next";
import { notFound } from "next/navigation";

type Params = {
  slug: string[];
};

function joinSlug(params: Params): string {
  return params.slug.join("/");
}

export async function generateMetadata({
  params,
}: {
  params: Promise<Params>;
}): Promise<Metadata> {
  let slug = joinSlug(await params);
  let lesson = await getLesson(slug);

  return {
    title: lesson ? `${lesson.title} - Vika` : "Vika",
    description: lesson?.description ?? undefined,
  };
}

export default async function Page({
  params,
}: {
  params: Promise<Params>;
}) {
  let slug = joinSlug(await params);
  let lesson = await getLesson(slug);

  if (!lesson) {
    notFound();
  }

  let Content = await getLessonContent(slug);

  return (
    <SidebarLayoutContent
      breadcrumbs={
        <Breadcrumbs>
          <BreadcrumbHome />
          <BreadcrumbSeparator className="max-md:hidden" />
          <Breadcrumb href={`/#${lesson.module.id}`} className="max-md:hidden">
            {lesson.module.title}
          </Breadcrumb>
          <BreadcrumbSeparator />
          <Breadcrumb>{lesson.title}</Breadcrumb>
        </Breadcrumbs>
      }
    >
      <div className="mx-auto max-w-7xl">
        <div className="-mx-2 sm:-mx-4">
          {lesson.video && (
            <Video
              id="video"
              src={lesson.video.url}
              poster={lesson.video.thumbnail}
            />
          )}
        </div>
        <div className="mx-auto flex max-w-2xl gap-x-10 py-10 sm:py-14 lg:max-w-5xl">
          <div className="w-full flex-1">
            <div id="content" className="prose">
              <Content />
            </div>
            <div className="mt-16 border-t border-gray-200 pt-8 dark:border-white/10">
              {lesson.next ? (
                <NextPageLink
                  title={lesson.next.title}
                  description={lesson.next.description}
                  href={`/docs/${lesson.next.id}`}
                />
              ) : null}
            </div>
          </div>
          <div className="hidden w-66 lg:block">
            <TableOfContents contentId="content" />
          </div>
        </div>
      </div>
    </SidebarLayoutContent>
  );
}


