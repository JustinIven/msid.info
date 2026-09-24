export interface LookupResponse {
  query: {
    input: string;
    domain: string;
    username: string;
    usernameProvided: boolean;
  };
  tenantId: string;
  tenantName: string | null;
  defaultDomain: string | null;
  tenantEnvironment: {
    tenantRegionScope: string | null;
    tenantRegionSubScope: string | null;
    cloudInstanceDisplayName: string;
    cloudInstance: string | null;
    audienceUrn: string | null;
  };
  nameSpaceType: string | null;
  endpoints: {
    issuer: string | null;
    authorizationEndpoint: string | null;
    tokenEndpoint: string | null;
    graphHost: string | null;
    cloudGraphHost: string | null;
  };
  federationInfo: {
    brandName: string | null;
    protocol: string | null;
    globalVersion: number | null;
    authUrl: string | null;
    metadataUrl: string | null;
    activeAuthenticationUrl: string | null;
  };
  loginExperiences: {
    isSignupAllowed: boolean | null;
    locale: number | null;
    bannerLogo: string | null;
    tileLogo: string | null;
    tileDarkLogo: string | null;
    illustration: string | null;
    backgroundColor: string | null;
    boilerPlateText: string | null;
    userIdLabel: string | null;
    keepMeSignedInDisabled: boolean | null;
    useTransparentLightBox: boolean | null;
  };
  userInfo: {
    username: string | null;
    displayName: string | null;
    state: string | null;
    domainType: string | null;
    isManaged: boolean | null;
    throttleStatus: string | null;
    credentials: {
      preferredCredential: string | null;
      hasPassword: boolean | null;
      hasAccessPass: boolean | null;
      hasDesktopSso: boolean | null;
      hasRemoteNGC: boolean | null;
      hasFido: boolean | null;
      hasCertAuth: boolean | null;
      hasGoogleFederation: boolean | null;
      hasQrCodePin: boolean | null;
      otcNotAutoSent: boolean | null;
      parameters: Record<string, unknown | null>;
    };
    callMetadata: {
      longRunningTransactionPartition: string | null;
      region: string | null;
      scaleUnit: string | null;
      isLongRunningTransaction: boolean | null;
    };
  };
  domains: string[];
  raw: Record<string, unknown>;
  meta: {
    durationMs: number;
    cache: { status: string; ttlSeconds: number };
    degraded: string[];
    calls: UpstreamCall[];
  };
}

export interface UpstreamCall {
  name: string;
  method: string;
  url: string;
  status: number | null;
  durationMs: number;
  ok: boolean;
  error: string | null;
}

export interface Problem {
  type: string;
  title: string;
  status: number;
  detail: string;
}

export interface AppConfig {
  turnstileSitekey: string;
  turnstileEnabled: boolean;
  cacheTtlSeconds: number;
}
