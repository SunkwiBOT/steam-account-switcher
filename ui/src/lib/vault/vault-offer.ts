import type { VaultStatusDto } from "../bindings/VaultStatusDto";

export type VaultOfferGateType = {
  check: (vault: Pick<VaultStatusDto, "password_suggested">) => boolean;
};

export const createVaultOfferGate = (): VaultOfferGateType => {
  let offered = false;
  return {
    check: (vault) => {
      if (offered || !vault.password_suggested) return false;
      offered = true;
      return true;
    },
  };
};
