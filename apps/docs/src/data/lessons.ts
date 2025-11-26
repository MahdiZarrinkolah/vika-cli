export type Module = {
  id: string;
  title: string;
  description: string;
  lessons: Lesson[];
};

export type Lesson = {
  id: string;
  title: string;
  description: string;
  video: {
    thumbnail: string;
    duration: number;
    url: string;
  } | null;
};

export function getModules(): Module[] {
  return lessons;
}

export async function getLesson(
  slug: string,
): Promise<(Lesson & { module: Module; next: Lesson | null }) | null> {
  let module = lessons.find(({ lessons }) =>
    lessons.some(({ id }) => id === slug),
  );

  if (!module) {
    return null;
  }

  let index = module.lessons.findIndex(({ id }) => id === slug);

  return {
    ...module.lessons[index],
    module,
    next: index < module.lessons.length - 1 ? module.lessons[index + 1] : null,
  };
}

export async function getLessonContent(slug: string) {
  return (await import(`@/data/lessons/${slug}.mdx`)).default;
}

const lessons = [
  {
    id: "getting-started",
    title: "Getting Started",
    description: "Learn the basics of Vika and get up and running quickly.",
    lessons: [
      {
        id: "getting-started",
        title: "Getting Started",
        description: "Introduction to Vika and its core concepts.",
        video: null,
      },
    ],
  },
  {
    id: "installation",
    title: "Installation",
    description: "Install and set up Vika in your project.",
    lessons: [
      {
        id: "installation",
        title: "Installation",
        description: "Step-by-step installation guide for Vika.",
        video: null,
      },
    ],
  },
  {
    id: "configuration",
    title: "Configuration",
    description: "Configure Vika to match your project's needs.",
    lessons: [
      {
        id: "configuration",
        title: "Configuration",
        description: "Learn how to configure Vika for your use case.",
        video: null,
      },
    ],
  },
  {
    id: "templates",
    title: "Templates",
    description: "Customize and create templates for code generation.",
    lessons: [
      {
        id: "templates",
        title: "Templates",
        description: "Understanding and working with Vika templates.",
        video: null,
      },
    ],
  },
  {
    id: "runtime",
    title: "Runtime",
    description: "Runtime utilities and HTTP client configuration.",
    lessons: [
      {
        id: "runtime",
        title: "Runtime",
        description: "Runtime configuration and HTTP client setup.",
        video: null,
      },
    ],
  },
  {
    id: "hooks",
    title: "Hooks",
    description: "Generate React hooks for data fetching.",
    lessons: [
      {
        id: "hooks",
        title: "Hooks",
        description: "Generate React Query and SWR hooks with Vika.",
        video: null,
      },
    ],
  },
  {
    id: "mocks",
    title: "Mocks",
    description: "Generate mock data for testing and development.",
    lessons: [
      {
        id: "mocks",
        title: "Mocks",
        description: "Create mock data generators for your APIs.",
        video: null,
      },
    ],
  },
  {
    id: "diff-mode",
    title: "Diff Mode",
    description: "Review and manage code generation diffs.",
    lessons: [
      {
        id: "diff-mode",
        title: "Diff Mode",
        description: "Use diff mode to review generated code changes.",
        video: null,
      },
    ],
  },
];
