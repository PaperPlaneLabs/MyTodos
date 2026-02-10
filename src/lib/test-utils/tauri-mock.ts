import { vi } from 'vitest';

/**
 * Mock responses for Tauri invoke commands
 * Usage: mockTauriInvoke({ 'command_name': returnValue })
 */
export function mockTauriInvoke(mockResponses: Record<string, any>) {
  const invokeMock = vi.fn((cmd: string, args?: any) => {
    if (cmd in mockResponses) {
      const response = mockResponses[cmd];
      // If response is a function, call it with args
      if (typeof response === 'function') {
        return Promise.resolve(response(args));
      }
      return Promise.resolve(response);
    }
    return Promise.reject(new Error(`Unmocked Tauri command: ${cmd}`));
  });

  // Mock the Tauri API
  vi.mock('@tauri-apps/api/core', () => ({
    invoke: invokeMock
  }));

  return invokeMock;
}

/**
 * Mock a Tauri command to throw an error
 */
export function mockTauriError(command: string, error: Error | string) {
  return mockTauriInvoke({
    [command]: () => {
      throw typeof error === 'string' ? new Error(error) : error;
    }
  });
}

/**
 * Reset all Tauri mocks
 */
export function resetTauriMocks() {
  vi.resetAllMocks();
}

/**
 * Create a mock project for testing
 */
export function createMockProject(overrides = {}) {
  return {
    id: 1,
    name: 'Test Project',
    color: '#6366f1',
    position: 0,
    created_at: '2024-01-01T00:00:00Z',
    total_time_seconds: 0,
    ...overrides
  };
}

/**
 * Create a mock task for testing
 */
export function createMockTask(overrides = {}) {
  return {
    id: 1,
    project_id: 1,
    section_id: null,
    title: 'Test Task',
    description: null,
    is_completed: false,
    position: 0,
    deadline: null,
    created_at: '2024-01-01T00:00:00Z',
    total_time_seconds: 0,
    ...overrides
  };
}

/**
 * Create a mock section for testing
 */
export function createMockSection(overrides = {}) {
  return {
    id: 1,
    project_id: 1,
    name: 'Test Section',
    position: 0,
    created_at: '2024-01-01T00:00:00Z',
    total_time_seconds: 0,
    ...overrides
  };
}

/**
 * Create a mock time entry for testing
 */
export function createMockTimeEntry(overrides = {}) {
  return {
    id: 1,
    task_id: 1,
    started_at: '2024-01-01T10:00:00Z',
    ended_at: '2024-01-01T11:00:00Z',
    duration_seconds: 3600,
    notes: null,
    ...overrides
  };
}

/**
 * Create a mock active timer for testing
 */
export function createMockActiveTimer(overrides = {}) {
  return {
    id: 1,
    task_id: 1,
    started_at: '2024-01-01T10:00:00Z',
    elapsed_seconds: 0,
    is_paused: false,
    ...overrides
  };
}
