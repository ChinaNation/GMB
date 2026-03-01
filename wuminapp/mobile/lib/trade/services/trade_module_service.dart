import 'package:wuminapp_mobile/trade/models/trade_models.dart';

class TradeModuleService {
  const TradeModuleService();

  List<TradeCapability> capabilities() {
    return const [
      TradeCapability(
        stage: TradeStage.onchainPhase1,
        title: '链上交易',
        description: '交易构建、签名、广播、状态追踪',
        enabled: true,
      ),
      TradeCapability(
        stage: TradeStage.offchainPhase2,
        title: '链下交易',
        description: '账本、撮合、清结算、自动对账',
        enabled: false,
      ),
    ];
  }
}
