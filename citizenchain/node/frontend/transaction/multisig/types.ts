export type AdminWalletMatch = {
  address: string;
  pubkeyHex: string;
  walletLabel: string;
};

export type VoteSignRequestResult = {
  requestJson: string;
  requestId: string;
  expectedPayloadHash: string;
  signNonce: number;
  signBlockNumber: number;
};

export type VoteSubmitResult = {
  txHash: string;
};

export type TransferProposalDetail = {
  proposalId: number;
  actorCidNumber: string | null;
  fundingAccountHex: string;
  beneficiaryHex: string;
  amountFen: string;
  remark: string;
  proposerHex: string;
};

export type SweepProposalDetail = {
  proposalId: number;
  actorCidNumber: string;
  institutionAccountHex: string;
  amountFen: string;
};

export type SafetyFundProposalDetail = {
  proposalId: number;
  actorCidNumber: string;
  institutionAccountHex: string;
  beneficiaryHex: string;
  amountFen: string;
  remark: string;
};

export type MultisigTransferProposalDetails = {
  transferDetail: TransferProposalDetail | null;
  safetyFundDetail: SafetyFundProposalDetail | null;
  sweepDetail: SweepProposalDetail | null;
};
