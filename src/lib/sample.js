// Sample data used when running in a plain browser (npm run dev without Tauri).
// In the real desktop app these come from the markdown vault via Rust commands.

export const SAMPLE_PEOPLE = [
  conv("marcus-koh", "Marcus Koh", "Frontend Engineer", "#6b7d9c", "2026-06-16", [
    {
      date: "2026-06-16",
      title: "Roadmap & career growth",
      body: "Walked through Q3 roadmap ownership. Marcus wants to lead the design-system migration — a strong signal that more scope is the right next step.",
      actions: [
        { text: "Share migration RFC by Fri", done: false },
        { text: "Set up intro with platform team lead", done: false },
      ],
    },
    {
      date: "2026-06-02",
      title: "Post-launch retro",
      body: "Checkout v2 launch went smoothly. He flagged on-call fatigue during the crunch week, so watch workload before the next cycle.",
      actions: [{ text: "Rebalance on-call rotation", done: true }],
    },
    {
      date: "2026-05-19",
      title: "Quarter kickoff",
      body: "Set goals for the quarter: ship checkout v2, mentor Liam, and lift test coverage above 80 percent.",
      actions: [],
    },
  ]),
  conv("ana-reyes", "Ana Reyes", "Product Design", "#a86b5a", "2026-06-15", [
    {
      date: "2026-06-15",
      title: "Portfolio & promo case",
      body: "Reviewed her promo packet. Strong systems work; one more cross-team project would make the case airtight.",
      actions: [
        { text: "Review portfolio draft", done: false },
        { text: "Find a cross-team project", done: false },
      ],
    },
  ]),
  conv("theo-okafor", "Theo Okafor", "Brand Marketing", "#8d6480", "2026-05-25", [
    {
      date: "2026-05-25",
      title: "Campaign retro",
      body: "Spring campaign underperformed. We agreed on clearer goals and a weekly check-in to rebuild momentum.",
      actions: [
        { text: "Set weekly check-in", done: false },
        { text: "Align on Q3 campaign goals", done: false },
      ],
    },
  ]),
  bare("sofia-petrov", "Sofia Petrov", "Backend Engineer", "#a8824f", "2026-06-12"),
  bare("liam-walsh", "Liam Walsh", "Design Intern", "#5d8a8a", "2026-06-16"),
  bare("jana-diaz", "Jana Diaz", "Data Science", "#a8824f", "2026-06-09"),
  bare("grace-lin", "Grace Lin", "Product Manager", "#7a8b5a", "2026-06-07"),
  bare("priya-nair", "Priya Nair", "Sales Engineer", "#6f8f72", "2026-05-17"),
  bare("diego-santos", "Diego Santos", "Support Lead", "#a8824f", "2026-05-30"),
];

export const SAMPLE_TASKS = [
  task("Share migration RFC with platform team", "Marcus Koh", "Fri", "high", "todo"),
  task("Intro Marcus to platform team lead", "Marcus Koh", "Next wk", "low", "todo"),
  task("Review Ana portfolio draft", "Ana Reyes", "Today", "med", "todo"),
  task("Draft Theo check-in agenda", "Theo Okafor", "Today", "high", "todo"),
  task("Send Jana data-eng feedback", "Jana Diaz", "Thu", "med", "todo"),
  task("Book team offsite venue", "", "Jul 1", "low", "todo"),
  task("Rebalance on-call rotation", "Marcus Koh", "Jun 20", "med", "doing"),
  task("Prep Q3 headcount plan", "", "Jun 19", "high", "doing"),
  task("Finalize Marcus promo packet", "Marcus Koh", "Jun 25", "high", "doing"),
  task("Approve Sofia PTO request", "Sofia Petrov", "", "low", "done", true, "2026-06-19"),
  task("Publish release notes", "", "", "med", "done", true, "2026-06-19"),
  task("Review Kenji's PR", "Kenji Mori", "", "med", "done", true, "2026-06-18"),
  task("Sync with Grace on roadmap", "Grace Lin", "", "high", "done", true, "2026-06-18"),
  task("Draft offsite agenda", "", "", "low", "done", true, "2026-06-16"),
  task("Close Q2 budget", "", "", "high", "done", true, "2026-06-07"),
  task("Migrate old tickets", "", "", "low", "done", true, "2026-05-09", true),
  task("Update onboarding doc", "Liam Walsh", "", "low", "done", true, "2026-04-28", true),
];

// ---- helpers -------------------------------------------------------------

function conv(slug, name, role, color, lastMet, conversations) {
  return {
    slug,
    name,
    role,
    bio: "",
    color,
    group: "Direct reports",
    last_met: lastMet,
    conversations,
  };
}

function bare(slug, name, role, color, lastMet) {
  return conv(slug, name, role, color, lastMet, []);
}

function task(title, person, due, priority, column, done = false, completed_at = "", archived = false) {
  return { title, person, due, priority, column, done, completed_at, archived };
}
