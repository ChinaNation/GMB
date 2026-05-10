import 'package:wuminapp_mobile/governance/admins-change/models/admin_subject.dart';
import 'package:wuminapp_mobile/governance/admins-change/services/admin_subject_service.dart';

class AdminSetChangeController {
  AdminSetChangeController({AdminSubjectService? subjectService})
      : _subjectService = subjectService ?? AdminSubjectService();

  final AdminSubjectService _subjectService;

  Future<AdminSubjectState?> loadSubject(AdminSubjectIdentity identity) {
    return _subjectService.fetchByIdentity(identity);
  }
}
