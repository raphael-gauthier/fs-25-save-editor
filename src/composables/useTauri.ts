import { invoke } from "@tauri-apps/api/core";
import type { LocalizedMessage } from "@/lib/types";

export class TauriError extends Error {
  code: string;
  params: Record<string, string>;

  constructor(code: string, params: Record<string, string> = {}) {
    super(code);
    this.code = code;
    this.params = params;
  }
}

export function translateError(
  t: (key: string, params?: Record<string, string>) => string,
  error: unknown,
): string {
  if (error instanceof TauriError) {
    return t(error.code, error.params);
  }
  if (error instanceof Error) {
    return error.message;
  }
  return t("errors.unexpected");
}

function isLocalizedMessage(
  value: unknown,
): value is LocalizedMessage {
  return (
    typeof value === "object" &&
    value !== null &&
    "code" in value &&
    typeof (value as LocalizedMessage).code === "string"
  );
}

export function useTauri() {
  async function invokeCommand<T>(
    command: string,
    args?: Record<string, unknown>,
  ): Promise<T> {
    try {
      return await invoke<T>(command, args);
    } catch (error: unknown) {
      if (isLocalizedMessage(error)) {
        throw new TauriError(
          error.code,
          error.params ?? {},
        );
      }
      if (typeof error === "string") {
        throw new TauriError("errors.unexpected", { message: error });
      }
      if (error && typeof error === "object" && "message" in error) {
        throw new TauriError("errors.unexpected", {
          message: (error as { message: string }).message,
        });
      }
      throw new TauriError("errors.unexpected");
    }
  }

  return { invokeCommand };
}
