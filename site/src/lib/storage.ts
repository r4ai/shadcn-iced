// localStorage, which can be missing or throw (private windows, blocked
// site data): the choices then last until the page is left.

export function read(key: string) {
  try {
    return localStorage.getItem(key);
  } catch {
    return null;
  }
}

export function write(key: string, value: string) {
  try {
    localStorage.setItem(key, value);
  } catch {
    // Ignored, see above.
  }
}
