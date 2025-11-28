import { randomBytes, createHash, randomUUID } from "node:crypto";
import * as protocol from "../protocol";

export enum RedirectMethod {
  Web = "web",
  App = "app",
  AppURI = "app_uri",
}

type TokenResponse = {
  access_token: string;
  refresh_token?: string;
  expires_in?: number;
  scope?: string;
  token_type?: string;
};

type TokenSetOptions = {
  accessToken: string;
  refreshToken?: string;
  idToken?: string;
  expiresIn?: number;
  scope?: string;
};

class TokenSet {
  public readonly accessToken: string;
  public readonly refreshToken?: string;
  public readonly idToken?: string;
  public readonly scope?: string;
  private readonly expiresAt?: number;

  constructor(options: TokenSetOptions & { expiresAt?: number }) {
    this.accessToken = options.accessToken;
    this.refreshToken = options.refreshToken;
    this.idToken = options.idToken;
    this.scope = options.scope;
    this.expiresAt = options.expiresAt;
  }

  public isExpired(): boolean {
    if (!this.expiresAt) {
      return false;
    }
    return Date.now() >= this.expiresAt;
  }
}

type Options = {
  redirectMethod: RedirectMethod;
  // TODO
  providerIcon?: unknown;
  providerId?: string;
  description?: string;
};

const base64URLEncode = (buffer: Buffer) => {
  return buffer
    .toString("base64")
    .replace(/\+/g, "-")
    .replace(/\//g, "_")
    .replace(/=/g, "");
};

const generateChallenge = () => {
  const verifier = base64URLEncode(randomBytes(32));
  const challenge = base64URLEncode(
    createHash("sha256").update(Buffer.from(verifier)).digest()
  );

  return {
    codeChallenge: challenge,
    codeVerifier: verifier,
  };
};

export class PKCEClient {
  constructor(private options: Options) {}

  public async authorizationRequest(options: {
    endpoint: string;
    clientId: string;
    scope: string;
    extraParameters?: { [key: string]: string };
  }): Promise<{
    codeChallenge: string;
    codeVerifier: string;
    state: string;
    redirectURI: string;
    toURL: () => string;
  }> {
    const { codeChallenge, codeVerifier } = generateChallenge();

    // TODO: figure out what is required in here
    const state = btoa(
      JSON.stringify({
        providerName: "temp value",
        id: randomUUID(),
        flavor: "release",
      })
    );

    let redirectURI = "";
    switch (this.options.redirectMethod) {
      case RedirectMethod.App:
        redirectURI = "raycast://oauth?package_name=Extension";
        break;
      case RedirectMethod.AppURI:
        redirectURI = ""; // TODO: what does this mean
        break;
      case RedirectMethod.Web:
        redirectURI = "https://raycast.com/redirect?packageName=Extension";
        break;
    }

    const params = new URLSearchParams();
    params.append("client_id", options.clientId);
    params.append("redirect_uri", redirectURI);
    params.append("response_type", "code");
    params.append("scope", options.scope);
    params.append("code_challenge", codeChallenge);
    params.append("code_challenge_method", "S256");
    params.append("state", state);
    if (options.extraParameters) {
      for (const key in options.extraParameters) {
        params.set(key, options.extraParameters[key]!);
      }
    }
    const url = options.endpoint + "?" + params.toString();

    return {
      codeChallenge,
      codeVerifier,
      state,
      redirectURI,
      toURL: () => url,
    };
  }

  public async authorize(
    options:
      | {
          url: string;
        }
      | { toURL: () => string }
  ): Promise<{ authorizationCode: string }> {
    const url = "url" in options ? options.url : options.toURL();

    const parsedUrl = new URL(url);
    const state = parsedUrl.searchParams.get("state") ?? "";

    return protocol.oauthAuthorize(url, state);
  }

  public async setTokens(
    options: TokenSetOptions | TokenResponse
  ): Promise<void> {
    const providerId = this.options.providerId ?? "default";
    const isTokenResponse = "access_token" in options;

    const stored = {
      accessToken: isTokenResponse ? options.access_token : options.accessToken,
      refreshToken: isTokenResponse
        ? options.refresh_token
        : options.refreshToken,
      idToken: isTokenResponse ? undefined : options.idToken,
      scope: isTokenResponse ? options.scope : options.scope,
      expiresAt: undefined as number | undefined,
    };

    const expiresIn = isTokenResponse ? options.expires_in : options.expiresIn;
    if (expiresIn) {
      stored.expiresAt = Date.now() + expiresIn * 1000;
    }

    await protocol.oauthSetTokens(providerId, JSON.stringify(stored));
  }

  public async getTokens(): Promise<TokenSet | undefined> {
    const providerId = this.options.providerId ?? "default";
    const data = await protocol.oauthGetTokens(providerId);

    if (!data) {
      return undefined;
    }

    const parsed = JSON.parse(data);
    return new TokenSet(parsed);
  }

  public async removeTokens(): Promise<void> {
    const providerId = this.options.providerId ?? "default";
    await protocol.oauthRemoveTokens(providerId);
  }
}
