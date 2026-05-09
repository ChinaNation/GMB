export type AdminWalletMatch = {
  address: string;
  pubkeyHex: string;
  name: string;
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
  institutionHex: string;
  beneficiaryHex: string;
  amountFen: string;
  remark: string;
  proposerHex: string;
};

export type SweepProposalDetail = {
  proposalId: number;
  institutionHex: string;
  amountFen: string;
};

export type SafetyFundProposalDetail = {
  proposalId: number;
  beneficiaryHex: string;
  amountFen: string;
  remark: string;
};

export type DuoqianTransferProposalDetails = {
  transferDetail: TransferProposalDetail | null;
  safetyFundDetail: SafetyFundProposalDetail | null;
  sweepDetail: SweepProposalDetail | null;
};
