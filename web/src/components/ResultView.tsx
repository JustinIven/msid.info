import { useState } from "react";
import type { LookupResponse } from "../types";
import { Badge, type Tone } from "./Badge";
import { CopyButton } from "./CopyButton";
import { CredentialMatrix } from "./CredentialMatrix";
import { DomainList } from "./DomainList";
import { JsonTree } from "./JsonTree";
import { KeyValue } from "./KeyValue";
import { Panel } from "./Panel";
import { RawPane } from "./RawPane";

const TABS = ["overview", "domains", "user", "raw"] as const;
type Tab = (typeof TABS)[number];

function namespaceTone(value: string | null): Tone {
  if (value === "Federated") return "magenta";
  if (value === "Managed") return "accent";
  return "neutral";
}

function stateTone(value: string | null): Tone {
  if (value === "Exists" || value === "ExistsBothIDPs") return "ok";
  if (value === "NotExist") return "danger";
  if (value === "Throttled") return "warn";
  return "neutral";
}

function Header({ data }: { data: LookupResponse }) {
  return (
    <div className="panel px-3 py-3">
      <div className="flex flex-wrap items-baseline gap-x-3 gap-y-1">
        <h1 className="text-lg text-ink-bright">
          {data.tenantName ?? data.defaultDomain ?? data.query.domain}
        </h1>
        <Badge tone={namespaceTone(data.nameSpaceType)}>
          {data.nameSpaceType ?? "unknown"}
        </Badge>
        <Badge tone="neutral">{data.tenantEnvironment.cloudInstanceDisplayName}</Badge>
        {data.tenantEnvironment.tenantRegionScope && (
          <Badge tone="neutral">{data.tenantEnvironment.tenantRegionScope}</Badge>
        )}
      </div>
      <div className="mt-1.5 flex items-center gap-2">
        <code className="break-all text-accent">{data.tenantId}</code>
        <CopyButton value={data.tenantId} label="Copy tenant ID" />
      </div>
    </div>
  );
}

function Overview({ data }: { data: LookupResponse }) {
  const { tenantEnvironment: env, federationInfo: fed, endpoints, loginExperiences: ux } = data;

  return (
    <div className="grid gap-3 lg:grid-cols-2">
      <Panel title="tenant">
        <KeyValue
          rows={[
            { key: "tenantId", value: data.tenantId },
            { key: "tenantName", value: data.tenantName },
            {
              key: "defaultDomain",
              value: data.defaultDomain,
              hint: "Requires a configured Microsoft Graph app registration",
            },
            { key: "nameSpaceType", value: data.nameSpaceType },
            { key: "cloudInstance", value: env.cloudInstance },
            { key: "cloudInstanceDisplayName", value: env.cloudInstanceDisplayName },
            { key: "tenantRegionScope", value: env.tenantRegionScope },
            { key: "tenantRegionSubScope", value: env.tenantRegionSubScope },
            { key: "audienceUrn", value: env.audienceUrn },
          ]}
        />
      </Panel>

      <Panel title="federation">
        <KeyValue
          rows={[
            { key: "brandName", value: fed.brandName },
            { key: "protocol", value: fed.protocol },
            { key: "globalVersion", value: fed.globalVersion },
            { key: "authUrl", value: fed.authUrl },
            { key: "metadataUrl", value: fed.metadataUrl },
            { key: "activeAuthenticationUrl", value: fed.activeAuthenticationUrl },
          ]}
        />
      </Panel>

      <Panel title="endpoints">
        <KeyValue
          rows={[
            { key: "issuer", value: endpoints.issuer },
            { key: "authorizationEndpoint", value: endpoints.authorizationEndpoint },
            { key: "tokenEndpoint", value: endpoints.tokenEndpoint },
            { key: "graphHost", value: endpoints.graphHost },
            { key: "cloudGraphHost", value: endpoints.cloudGraphHost },
          ]}
        />
      </Panel>

      <Panel title="sign-in branding">
        <KeyValue
          rows={[
            { key: "isSignupAllowed", value: ux.isSignupAllowed },
            { key: "locale", value: ux.locale },
            { key: "userIdLabel", value: ux.userIdLabel },
            { key: "boilerPlateText", value: ux.boilerPlateText },
            {
              key: "backgroundColor",
              value: ux.backgroundColor,
              render: ux.backgroundColor ? (
                <span className="inline-flex items-center gap-2">
                  <span
                    aria-hidden
                    className="inline-block size-3 border border-line-bright"
                    style={{ background: ux.backgroundColor }}
                  />
                  <span className="text-ink-bright">{ux.backgroundColor}</span>
                </span>
              ) : undefined,
            },
            { key: "bannerLogo", value: ux.bannerLogo },
            { key: "tileLogo", value: ux.tileLogo },
            { key: "tileDarkLogo", value: ux.tileDarkLogo },
            { key: "illustration", value: ux.illustration },
            { key: "keepMeSignedInDisabled", value: ux.keepMeSignedInDisabled },
            { key: "useTransparentLightBox", value: ux.useTransparentLightBox },
          ]}
        />
      </Panel>
    </div>
  );
}

function UserPane({ data }: { data: LookupResponse }) {
  const { userInfo } = data;
  const parameters = Object.entries(userInfo.credentials.parameters).filter(
    ([, value]) => value !== null && value !== undefined,
  );

  return (
    <div className="space-y-3">
      <Panel
        title={`probed identity · ${data.query.username}`}
        actions={
          data.query.usernameProvided ? undefined : (
            <Badge tone="warn" title="No UPN was supplied, so admin@domain was probed">
              synthesized
            </Badge>
          )
        }
      >
        <KeyValue
          rows={[
            { key: "username", value: userInfo.username },
            { key: "displayName", value: userInfo.displayName },
            {
              key: "state",
              value: userInfo.state,
              render: userInfo.state ? (
                <Badge tone={stateTone(userInfo.state)}>{userInfo.state}</Badge>
              ) : undefined,
            },
            { key: "domainType", value: userInfo.domainType },
            { key: "isManaged", value: userInfo.isManaged },
            { key: "throttleStatus", value: userInfo.throttleStatus },
            { key: "preferredCredential", value: userInfo.credentials.preferredCredential },
            { key: "otcNotAutoSent", value: userInfo.credentials.otcNotAutoSent },
          ]}
        />
      </Panel>

      <Panel title="credential methods">
        <CredentialMatrix data={data} />
      </Panel>

      <Panel title="call metadata">
        <KeyValue
          rows={[
            { key: "region", value: userInfo.callMetadata.region },
            { key: "scaleUnit", value: userInfo.callMetadata.scaleUnit },
            {
              key: "longRunningTransactionPartition",
              value: userInfo.callMetadata.longRunningTransactionPartition,
            },
            {
              key: "isLongRunningTransaction",
              value: userInfo.callMetadata.isLongRunningTransaction,
            },
          ]}
        />
      </Panel>

      {parameters.length > 0 && (
        <Panel title="credential parameters">
          <JsonTree data={Object.fromEntries(parameters)} />
        </Panel>
      )}
    </div>
  );
}

interface Props {
  data: LookupResponse;
  onLookup: (query: string) => void;
}

export function ResultView({ data, onLookup }: Props) {
  const [tab, setTab] = useState<Tab>("overview");

  return (
    <div className="space-y-3">
      <Header data={data} />

      <div role="tablist" aria-label="Result sections" className="flex border-b border-line">
        {TABS.map((name) => (
          <button
            key={name}
            role="tab"
            aria-selected={tab === name}
            onClick={() => setTab(name)}
            className={`-mb-px border-b-2 px-3 py-1.5 text-[12px] uppercase tracking-[0.12em] ${
              tab === name
                ? "border-accent text-accent"
                : "border-transparent text-muted hover:text-ink"
            }`}
          >
            {name}
            {name === "domains" && (
              <span className="ml-1.5 text-muted">{data.domains.length}</span>
            )}
          </button>
        ))}
        <div className="flex flex-1 items-center justify-end pb-1">
          <CopyButton value={JSON.stringify(data, null, 2)} label="Copy full JSON" />
        </div>
      </div>

      {tab === "overview" && <Overview data={data} />}
      {tab === "domains" && <DomainList domains={data.domains} onSelect={onLookup} />}
      {tab === "user" && <UserPane data={data} />}
      {tab === "raw" && <RawPane data={data} />}
    </div>
  );
}
