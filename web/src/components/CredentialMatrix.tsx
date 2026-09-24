import type { LookupResponse } from "../types";

const LABELS: Array<[keyof LookupResponse["userInfo"]["credentials"], string]> = [
  ["hasPassword", "password"],
  ["hasFido", "fido2 / passkey"],
  ["hasRemoteNGC", "ms authenticator"],
  ["hasAccessPass", "temporary access pass"],
  ["hasCertAuth", "certificate"],
  ["hasQrCodePin", "qr code + pin"],
  ["hasDesktopSso", "seamless sso"],
  ["hasGoogleFederation", "google federation"],
];

function Cell({ label, state }: { label: string; state: boolean | null }) {
  const tone =
    state === null
      ? "border-line text-muted/60"
      : state
        ? "border-ok/40 bg-ok/5 text-ok"
        : "border-line text-muted";
  const mark = state === null ? "?" : state ? "●" : "○";
  return (
    <div className={`flex items-center gap-2 border px-2.5 py-1.5 ${tone}`}>
      <span aria-hidden>{mark}</span>
      <span className="truncate">{label}</span>
      <span className="sr-only">
        {state === null ? "unknown" : state ? "available" : "not available"}
      </span>
    </div>
  );
}

export function CredentialMatrix({ data }: { data: LookupResponse }) {
  const credentials = data.userInfo.credentials;
  return (
    <div className="grid grid-cols-1 gap-1.5 p-3 sm:grid-cols-2 lg:grid-cols-4">
      {LABELS.map(([key, label]) => (
        <Cell key={key} label={label} state={credentials[key] as boolean | null} />
      ))}
    </div>
  );
}
