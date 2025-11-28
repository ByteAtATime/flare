import { randomBytes, createHash } from "node:crypto";

export enum RedirectMethod {
  Web = "web",
  App = "app",
  AppURI = "app_uri",
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

export class PCKEClient {
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

    const state = base64URLEncode(randomBytes(16)); // i have no idea how oauth works please help

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
}
