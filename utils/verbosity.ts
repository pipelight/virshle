// Global
const VERBOSITY = { value: 0 };

// Getter/Setter
export const verbosity = {
  get: () => {
    return VERBOSITY.value;
  },
  set: (n: number) => {
    VERBOSITY.value = n;
  },
};
