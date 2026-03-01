enum TradeStage {
  onchainPhase1,
  offchainPhase2,
}

class TradeCapability {
  const TradeCapability({
    required this.stage,
    required this.title,
    required this.description,
    required this.enabled,
  });

  final TradeStage stage;
  final String title;
  final String description;
  final bool enabled;
}
