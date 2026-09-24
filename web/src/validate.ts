/// Mirrors `src/validate.rs` so the UI can flag bad input before spending a request.
/// The server revalidates regardless; this is purely a convenience.

const LABEL = /^[a-z0-9]([a-z0-9-]*[a-z0-9])?$/;
const LOCAL_PART = /^[a-z0-9._+'-]{1,64}$/;

export function normalize(input: string): string {
  return input.trim().replace(/^\.+|\.+$/g, "").toLowerCase();
}

export function validationError(rawInput: string): string | null {
  const input = normalize(rawInput);
  if (!input) return null;
  if (input.length > 320) return "Query is longer than 320 characters.";
  // eslint-disable-next-line no-control-regex
  if (/[^\x00-\x7F]/.test(input))
    return "Convert internationalized domains to punycode (xn--...) first.";

  const parts = input.split("@");
  if (parts.length > 2) return "Query must contain at most one '@'.";

  let domain = input;
  if (parts.length === 2) {
    const [local, rest] = parts;
    if (!LOCAL_PART.test(local))
      return "The part before '@' contains unsupported characters.";
    domain = rest;
  }

  const labels = domain.split(".");
  if (labels.length < 2) return "Enter a domain such as contoso.com.";
  if (domain.length > 253) return "Domain is too long.";
  if (!labels.every((label) => label.length <= 63 && LABEL.test(label)))
    return "Domain may only contain letters, digits, hyphens and dots.";
  if (/^\d+$/.test(labels[labels.length - 1]))
    return "IP addresses are not valid lookup targets.";

  return null;
}
