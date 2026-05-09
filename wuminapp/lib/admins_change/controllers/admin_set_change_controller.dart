import 'package:wuminapp_mobile/admins_change/models/admin_subject.dart';
import 'package:wuminapp_mobile/admins_change/services/admin_subject_service.dart';

class AdminSetChangeController {
  AdminSetChangeController({AdminSubjectService? subjectService})
      : _subjectService = subjectService ?? AdminSubjectService();

  final AdminSubjectService _subjectService;

  Future<AdminSubjectState?> loadSubject(String institutionIdentity) {
    return _subjectService.fetchByInstitutionIdentity(institutionIdentity);
  }
}
