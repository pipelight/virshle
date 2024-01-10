declare global {
  let loggy: Console;
  interface Window {
    loggy: Console;
  }
}
const console_clone = JSON.parse(
  JSON.stringify(console),
);

const { log, error, warn, info, debug, trace } = console;
const backup = { log, error, warn, info, debug, trace };
window.loggy = backup;

// Global
const VERBOSITY = { value: 0 };

const disable = function () {
  // Disable console log statements
  for (const [key, _value] of Object.entries(backup)) {
    loggy[key] = () => undefined;
  }
};

// Enable console
const enable = function () {
  loggy = {
    ...loggy,
  };
};

// Getter/Setter
export const verbosity = {
  // Disable console
  get: () => {
    return VERBOSITY.value;
  },
  set: (n?: number) => {
    // Safe guard
    if (n == undefined) {
      n = 0;
    }
    VERBOSITY.value = n!;
    switch (n) {
      case 0:
        disable();
        loggy = {
          ...loggy,
          log,
          error,
        };
        break;
      case 1:
        disable();
        loggy = {
          ...loggy,
          log,
          error,
          warn,
          info,
        };
        break;
      case 2:
        disable();
        loggy = {
          ...loggy,
          log,
          error,
          warn,
          info,
          debug,
        };
        break;
      case 3 || 4:
        disable();
        loggy = {
          ...loggy,
          log,
          error,
          warn,
          info,
          debug,
          trace
        };
        break;
    }
  },
};
