export type Module = {
  id: string;
  title: string;
  description: string;
  docs: Doc[];
};

export type Doc = {
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
  return docs;
}

export async function getLesson(
  slug: string,
): Promise<(Doc & { module: Module; next: Doc | null }) | null> {
  let module = docs.find(({ docs }) =>
    docs.some(({ id }) => id === slug),
  );

  if (!module) {
    return null;
  }

  let index = module.docs.findIndex(({ id }) => id === slug);

  return {
    ...module.docs[index],
    module,
    next: index < module.docs.length - 1 ? module.docs[index + 1] : null,
  };
}

export async function getLessonContent(slug: string) {
  return (await import(`@/data/docs/${slug}.mdx`)).default;
}

const docs = [
  {
    id: "getting-started",
    title: "Getting Started",
    description: "Learn the basics of Vika and get up and running quickly.",
    docs: [
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
    docs: [
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
    docs: [
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
    docs: [
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
    docs: [
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
    docs: [
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
    docs: [
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
    docs: [
      {
        id: "diff-mode",
        title: "Diff Mode",
        description: "Use diff mode to review generated code changes.",
        video: null,
      },
    ],
  },
];
