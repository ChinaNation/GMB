import 'package:flutter/material.dart';

import 'package:citizenapp/8964/models/square_models.dart';
import 'package:citizenapp/ui/app_theme.dart';

class SquareFeedTabs extends StatelessWidget {
  const SquareFeedTabs({
    super.key,
    required this.selected,
    required this.onChanged,
  });

  final SquareFeedKind selected;
  final ValueChanged<SquareFeedKind> onChanged;

  @override
  Widget build(BuildContext context) {
    return SegmentedButton<SquareFeedKind>(
      showSelectedIcon: false,
      segments: SquareFeedKind.values
          .map(
            (kind) => ButtonSegment<SquareFeedKind>(
              value: kind,
              label: Text(kind.label),
            ),
          )
          .toList(growable: false),
      selected: {selected},
      onSelectionChanged: (values) => onChanged(values.first),
      style: ButtonStyle(
        visualDensity: VisualDensity.compact,
        foregroundColor: WidgetStateProperty.resolveWith((states) {
          if (states.contains(WidgetState.selected)) {
            return AppTheme.primary;
          }
          return AppTheme.textSecondary;
        }),
        side: WidgetStateProperty.all(
          const BorderSide(color: AppTheme.border),
        ),
      ),
    );
  }
}
