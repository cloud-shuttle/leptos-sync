export const TestEnvironments = {
  // Development environment
  dev: {
    baseURL: 'http://localhost:3000',
    timeout: 30000,
    retries: 1,
  },

  // Staging environment
  staging: {
    baseURL: 'https://staging.leptos-sync.example.com',
    timeout: 60000,
    retries: 2,
  },

  // Production environment
  production: {
    baseURL: 'https://leptos-sync.example.com',
    timeout: 90000,
    retries: 3,
  },

  // CI environment
  ci: {
    baseURL: 'http://localhost:3000',
    timeout: 120000,
    retries: 2,
    workers: 1,
  },
};

export const TestData = {
  // Sample data for testing
  sampleItems: [
    { id: 1, name: 'Test Item 1', description: 'First test item' },
    { id: 2, name: 'Test Item 2', description: 'Second test item' },
    { id: 3, name: 'Test Item 3', description: 'Third test item' },
  ],

  // Performance thresholds
  performance: {
    pageLoad: 3000,      // 3 seconds
    interaction: 100,     // 100ms
    syncOperation: 5000,  // 5 seconds
  },

  // Accessibility thresholds
  accessibility: {
    colorContrast: 4.5,   // WCAG AA standard
    focusVisible: true,
    keyboardNavigable: true,
  },
};
