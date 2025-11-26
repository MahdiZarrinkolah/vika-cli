export type Module = {
  id: string;
  title: string;
  description: string;
  docs: Doc[];
};

export type Doc = {
  /**
   * Slug relative to `apps/docs/content`, e.g. `getting-started/introduction`.
   * The page URL will be `/docs/${id}`.
   */
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
  // MDX files live under `apps/docs/content`.
  return (await import(`../../content/${slug}.mdx`)).default;
}

const docs: Module[] = [
  {
    id: "getting-started",
    title: "Getting Started",
    description: "Get up and running with Vika CLI.",
    docs: [
      {
        id: "getting-started/introduction",
        title: "Introduction",
        description: "What Vika is and how it fits into your toolchain.",
        video: null,
      },
      {
        id: "getting-started/installation",
        title: "Installation",
        description: "Install and set up Vika in your project.",
        video: null,
      },
      {
        id: "getting-started/quickstart",
        title: "Quickstart",
        description: "Generate your first client from an API spec.",
        video: null,
      },
    ],
  },
  {
    id: "cli",
    title: "CLI",
    description: "Command-line interface for running Vika operations.",
    docs: [
      {
        id: "cli/overview",
        title: "Overview",
        description: "Overview of the Vika CLI engine and usage patterns.",
        video: null,
      },
      {
        id: "cli/commands/generate",
        title: "generate",
        description: "Generate code from your API specifications.",
        video: null,
      },
      {
        id: "cli/commands/init",
        title: "init",
        description: "Initialize Vika configuration in a project.",
        video: null,
      },
      {
        id: "cli/commands/inspect",
        title: "inspect",
        description: "Inspect specs, templates, and generated outputs.",
        video: null,
      },
      {
        id: "cli/commands/update",
        title: "update",
        description: "Update generated code when specs change.",
        video: null,
      },
      {
        id: "cli/commands/diff",
        title: "diff",
        description: "Run the CLI in diff mode to review changes.",
        video: null,
      },
    ],
  },
  {
    id: "configuration",
    title: "Configuration",
    description: "Project-level configuration for Vika CLI.",
    docs: [
      {
        id: "configuration/overview",
        title: "Overview",
        description: "High-level overview of Vika configuration.",
        video: null,
      },
      {
        id: "configuration/vika-json",
        title: "vika.json schema",
        description: "Reference for the vika.json configuration schema.",
        video: null,
      },
      {
        id: "configuration/presets",
        title: "Presets",
        description: "Use and share reusable configuration presets.",
        video: null,
      },
      {
        id: "configuration/multi-spec",
        title: "Multi-spec",
        description: "Configure Vika to work with multiple API specs.",
        video: null,
      },
    ],
  },
  {
    id: "templates",
    title: "Template System",
    description: "How Vika templates drive code generation.",
    docs: [
      {
        id: "templates/overview",
        title: "Overview",
        description: "Conceptual overview of the template system.",
        video: null,
      },
      {
        id: "templates/built-in-templates",
        title: "Built-in Templates",
        description: "Explore the built-in templates that ship with Vika.",
        video: null,
      },
      {
        id: "templates/overriding",
        title: "Overriding Templates",
        description: "Override built-in templates in your own project.",
        video: null,
      },
      {
        id: "templates/writing-custom-templates",
        title: "Writing Custom Templates",
        description: "Author and organize your own custom templates.",
        video: null,
      },
    ],
  },
  {
    id: "runtime",
    title: "Runtime",
    description: "Runtime utilities and HTTP client abstractions.",
    docs: [
      {
        id: "runtime/overview",
        title: "Overview",
        description: "Runtime concepts and how generated code uses them.",
        video: null,
      },
      {
        id: "runtime/client",
        title: "HTTP Client",
        description: "HTTP client interface and integrations.",
        video: null,
      },
      {
        id: "runtime/api-result",
        title: "ApiResult",
        description: "Standardized result type for API calls.",
        video: null,
      },
      {
        id: "runtime/middleware",
        title: "Middleware",
        description: "Apply cross-cutting concerns around API calls.",
        video: null,
      },
      {
        id: "runtime/auth",
        title: "Auth",
        description: "Patterns for handling auth in generated clients.",
        video: null,
      },
    ],
  },
  {
    id: "generators",
    title: "Generators",
    description: "Output targets supported by Vika.",
    docs: [
      {
        id: "generators/typescript",
        title: "TypeScript Types",
        description: "Generate TypeScript type definitions from specs.",
        video: null,
      },
      {
        id: "generators/zod",
        title: "Zod Schemas",
        description: "Generate Zod schemas for runtime validation.",
        video: null,
      },
      {
        id: "generators/api-clients",
        title: "API Clients",
        description: "Generate HTTP clients for your APIs.",
        video: null,
      },
      {
        id: "generators/hooks/react-query",
        title: "React Query Hooks",
        description: "Generate React Query hooks for data fetching.",
        video: null,
      },
      {
        id: "generators/hooks/swr",
        title: "SWR Hooks",
        description: "Generate SWR hooks for data fetching.",
        video: null,
      },
    ],
  },
  {
    id: "mocks",
    title: "Mocking",
    description: "Mock servers and data for testing and development.",
    docs: [
      {
        id: "mocks/overview",
        title: "Overview",
        description: "Overview of Vika's mocking capabilities.",
        video: null,
      },
      {
        id: "mocks/msw",
        title: "MSW",
        description: "Integrate Vika with MSW for request mocking.",
        video: null,
      },
      {
        id: "mocks/mirage",
        title: "Mirage",
        description: "Use Mirage with Vika-generated mocks.",
        video: null,
      },
      {
        id: "mocks/factories",
        title: "Factories",
        description: "Define factories for generating mock entities.",
        video: null,
      },
      {
        id: "mocks/scenarios",
        title: "Scenarios",
        description: "Compose mock scenarios for complex flows.",
        video: null,
      },
    ],
  },
  {
    id: "diff",
    title: "Diff Mode",
    description: "Understand and manage diffs in generated code.",
    docs: [
      {
        id: "diff/overview",
        title: "Overview",
        description: "High-level overview of diff mode.",
        video: null,
      },
      {
        id: "diff/cli",
        title: "CLI Diff",
        description: "Use the CLI diff command effectively.",
        video: null,
      },
      {
        id: "diff/breaking-changes",
        title: "Breaking Changes",
        description:
          "Detect and reason about breaking changes in your specs.",
        video: null,
      },
      {
        id: "diff/compatibility",
        title: "Compatibility",
        description:
          "Compatibility guarantees and strategies around changes.",
        video: null,
      },
    ],
  },
  {
    id: "advanced",
    title: "Advanced",
    description: "Advanced usage, performance, and troubleshooting.",
    docs: [
      {
        id: "advanced/performance",
        title: "Performance",
        description: "Performance considerations and tuning tips.",
        video: null,
      },
      {
        id: "advanced/troubleshooting",
        title: "Troubleshooting",
        description: "Diagnose and fix common issues with Vika.",
        video: null,
      },
      {
        id: "advanced/faq",
        title: "FAQ",
        description: "Frequently asked questions about Vika CLI.",
        video: null,
      },
    ],
  },
];

