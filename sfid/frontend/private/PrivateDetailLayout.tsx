// 中文注释:私权机构详情页布局 — 顶部左右双板块 + 账户 + 资料库。
//
// 布局:
//   顶部一整块 Card(标题 = 机构名称,编辑/取消/保存按钮在 Card extra 右上角):
//     ┌ 左 Col:SFID 信息(只读)──────────────┐  ┌ 右 Col:机构信息 ──────────────┐
//     │ 身份ID / 省 / 市 / 盈利属性 / 机构(均纯中文) │  │ 机构名称 + 搜索查重图标       │
//     │ 创建时间 / 创建用户                  │  │ 私权类型/法人资格只读展示     │
//     │                                      │  │ 所属法人 AutoComplete(需挂靠 F)│
//     └──────────────────────────────────────┘  └───────────────────────────────┘
//
// 右板块交互:
//   默认态 = 只读 Descriptions 展示,右上角显示"编辑"按钮
//   编辑态 = Form 可操作,右上角切换为"取消" + "保存"
//   机构名称右侧搜索图标:输入后点击查重;重名则禁止保存;名称未改动视为已通过
//   需挂靠的非法人所属法人:输入后点搜索图标触发模糊搜索(/institution/search-parents)
//
// 账户列表(AccountList)每家机构自带"主账户"/"费用账户"两条默认账户,
// 创建后只登记在 SFID;链上状态由区块链软件同步回来。

import React, { useEffect, useState } from 'react';
import {
  Alert,
  AutoComplete,
  Button,
  Card,
  Col,
  Descriptions,
  Form,
  Input,
  Row,
  Space,
  Spin,
  Tag,
  Typography,
  Upload,
} from 'antd';
import { SearchOutlined, UploadOutlined } from '@ant-design/icons';
import type { AdminAuth } from '../auth/types';
import type { AdminActionType, AdminSecurityGrantOutput } from '../admins/admin_security_api';
import {
  INSTITUTION_CODE_LABEL,
  PARTNERSHIP_KIND_LABEL,
  PRIVATE_TYPE_LABEL,
} from '../subjects/labels';
import {
  checkInstitutionName,
  searchParentInstitutions,
  updateInstitution,
  uploadLegalRepresentativePhoto,
  type InstitutionDetail,
  type ParentInstitutionRow,
} from './common/api';
import { searchLegalRepresentativeCitizens } from '../citizens/api';
import { AccountList } from '../accounts/AccountList';
import { CreateAccountModal } from '../accounts/CreateAccountModal';
import { DocumentLibrary } from '../docs/DocumentLibrary';
import { notice } from '../utils/notice';

// 创建者角色中文映射(与列表页保持一致)。
const CREATED_BY_ROLE_LABEL: Record<string, string> = {
  FEDERAL_ADMIN: '联邦管理员',
  CITY_ADMIN: '市管理员',
};

interface Props {
  auth: AdminAuth;
  detail: InstitutionDetail;
  canWrite: boolean;
  loading: boolean;
  onReload: () => void;
  onDeleteAccount: (accountName: string) => void;
  createPasskeyChallengeGrant: (
    actionType: AdminActionType,
    payload: unknown,
  ) => Promise<AdminSecurityGrantOutput>;
}

interface InfoFormValues {
  institution_name: string;
  /** 需挂靠的非法人所属法人 sfid_number */
  parent_sfid_number?: string;
  legal_rep_name: string;
  legal_rep_sfid_number: string;
  legal_rep_photo_path: string;
  legal_rep_photo_name: string;
  legal_rep_photo_mime: string;
  legal_rep_photo_size?: number;
}

export const PrivateDetailLayout: React.FC<Props> = ({
  auth,
  detail,
  canWrite,
  loading,
  onReload,
  onDeleteAccount,
  createPasskeyChallengeGrant,
}) => {
  const inst = detail.institution;
  const accounts = detail.accounts;
  const [createAccountOpen, setCreateAccountOpen] = useState(false);

  // ── 右板块:编辑/只读切换 ──
  const [editing, setEditing] = useState(false);
  const [form] = Form.useForm<InfoFormValues>();
  const [savingInfo, setSavingInfo] = useState(false);

  // ── 机构名称查重状态 ──
  // null = 未查 / 未改名(视为 ok);true = 查重通过;false = 已占用
  const [nameChecking, setNameChecking] = useState(false);
  const [nameAvailable, setNameAvailable] = useState<boolean | null>(null);
  const [currentName, setCurrentName] = useState<string>(inst.institution_name ?? '');
  const [legalRepSearching, setLegalRepSearching] = useState(false);
  const [legalRepOptions, setLegalRepOptions] = useState<string[]>([]);
  const [photoUploading, setPhotoUploading] = useState(false);
  const [photoName, setPhotoName] = useState<string>(inst.legal_rep_photo_name ?? '');

  const isF = inst.subject_property === 'F';
  const requiresParent = isF && !['GT', 'GP'].includes(inst.institution_code);
  // 完善判断:名称必填;仅需挂靠的非法人要求 parent_sfid_number;私权类型由创建时编码确定。
  const needsCompletion =
    !inst.institution_name ||
    (requiresParent && !inst.parent_sfid_number) ||
    !inst.legal_rep_name ||
    !inst.legal_rep_sfid_number ||
    !inst.legal_rep_photo_path;

  // ── 需挂靠非法人所属法人搜索 ──
  const [parentSearchOpts, setParentSearchOpts] = useState<ParentInstitutionRow[]>([]);
  const [parentSearching, setParentSearching] = useState(false);
  // 当前选中的法人(用于展示已选项名称;首次进入若 inst.parent_sfid_number 有值,也要一次性拿到显示名)
  const [selectedParent, setSelectedParent] = useState<ParentInstitutionRow | null>(null);

  // detail 变更 → 需挂靠的非法人若有 parent_sfid_number,则拉一次展示名称。
  useEffect(() => {
    if (!requiresParent || !inst.parent_sfid_number) {
      setSelectedParent(null);
      return;
    }
    // 用 sfid_number 自身作为查询词反查名称(传机构自身落位省市,既有挂靠必然满足地域规则)
    let cancelled = false;
    searchParentInstitutions(auth, inst.parent_sfid_number, {
      fInstitution: inst.institution_code,
      province: inst.province,
      city: inst.city,
    })
      .then((rows) => {
        if (cancelled) return;
        const hit = rows.find((r) => r.sfid_number === inst.parent_sfid_number);
        setSelectedParent(hit ?? null);
      })
      .catch(() => {
        if (!cancelled) setSelectedParent(null);
      });
    return () => {
      cancelled = true;
    };
  }, [requiresParent, inst.parent_sfid_number, auth.access_token]);

  // 搜索(仅在用户点击搜索图标时触发,不自动 onSearch)
  const onParentSearch = async (value: string) => {
    const q = value.trim();
    if (!q) {
      notice.warning('请先输入 SFID 或机构名称');
      setParentSearchOpts([]);
      return;
    }
    setParentSearching(true);
    try {
      // 改挂与创建同源:后端按 subjects/uninorg 地域规则预过滤候选父级
      const rows = await searchParentInstitutions(auth, q, {
        fInstitution: inst.institution_code,
        province: inst.province,
        city: inst.city,
      });
      setParentSearchOpts(rows);
      if (rows.length === 0) {
        notice.info('未找到匹配的法人机构');
      }
    } catch (err) {
      notice.error(err, '');
      setParentSearchOpts([]);
    } finally {
      setParentSearching(false);
    }
  };

  const triggerParentSearch = () => {
    if (parentSearching) return;
    const q = (form.getFieldValue('parent_sfid_number') ?? '') as string;
    onParentSearch(q);
  };

  // detail 重新加载(保存成功后 onReload)→ 重置编辑态
  useEffect(() => {
    setEditing(false);
    setNameAvailable(null);
    setCurrentName(inst.institution_name ?? '');
    form.setFieldsValue({
      institution_name: inst.institution_name ?? '',
      parent_sfid_number: inst.parent_sfid_number ?? undefined,
      legal_rep_name: inst.legal_rep_name ?? '',
      legal_rep_sfid_number: inst.legal_rep_sfid_number ?? '',
      legal_rep_photo_path: inst.legal_rep_photo_path ?? '',
      legal_rep_photo_name: inst.legal_rep_photo_name ?? '',
      legal_rep_photo_mime: inst.legal_rep_photo_mime ?? '',
      legal_rep_photo_size: inst.legal_rep_photo_size ?? undefined,
    });
    setPhotoName(inst.legal_rep_photo_name ?? '');
    setLegalRepOptions([]);
  }, [
    inst.sfid_number,
    inst.institution_name,
    inst.parent_sfid_number,
    inst.legal_rep_name,
    inst.legal_rep_sfid_number,
    inst.legal_rep_photo_path,
  ]);

  const onClickEdit = () => {
    setEditing(true);
    setNameAvailable(null);
    form.setFieldsValue({
      institution_name: inst.institution_name ?? '',
      parent_sfid_number: inst.parent_sfid_number ?? undefined,
      legal_rep_name: inst.legal_rep_name ?? '',
      legal_rep_sfid_number: inst.legal_rep_sfid_number ?? '',
      legal_rep_photo_path: inst.legal_rep_photo_path ?? '',
      legal_rep_photo_name: inst.legal_rep_photo_name ?? '',
      legal_rep_photo_mime: inst.legal_rep_photo_mime ?? '',
      legal_rep_photo_size: inst.legal_rep_photo_size ?? undefined,
    });
    setPhotoName(inst.legal_rep_photo_name ?? '');
    setCurrentName(inst.institution_name ?? '');
  };

  const onClickCancel = () => {
    setEditing(false);
    setNameAvailable(null);
    form.setFieldsValue({
      institution_name: inst.institution_name ?? '',
      parent_sfid_number: inst.parent_sfid_number ?? undefined,
      legal_rep_name: inst.legal_rep_name ?? '',
      legal_rep_sfid_number: inst.legal_rep_sfid_number ?? '',
      legal_rep_photo_path: inst.legal_rep_photo_path ?? '',
      legal_rep_photo_name: inst.legal_rep_photo_name ?? '',
      legal_rep_photo_mime: inst.legal_rep_photo_mime ?? '',
      legal_rep_photo_size: inst.legal_rep_photo_size ?? undefined,
    });
    setPhotoName(inst.legal_rep_photo_name ?? '');
    setCurrentName(inst.institution_name ?? '');
  };

  const onNameInputChange = (e: React.ChangeEvent<HTMLInputElement>) => {
    const v = e.target.value;
    setCurrentName(v);
    // 名称改动 → 需要重新查重
    if (nameAvailable !== null) setNameAvailable(null);
  };

  const isNameUnchanged = () => {
    return currentName.trim() === (inst.institution_name ?? '').trim();
  };

  const onCheckName = async () => {
    const name = currentName.trim();
    if (!name) {
      notice.warning('请先输入机构名称');
      return;
    }
    if (isNameUnchanged()) {
      // 与原名一致,直接视为可用
      setNameAvailable(true);
      return;
    }
    setNameChecking(true);
    try {
      // 私权机构全国唯一查重(不传 subject_property/city 即走全国范围;后端会排除自身名不在此函数,
      // 所以必须在名称改动时才调用;未改名的场景已在 isNameUnchanged 提前返回)
      const { exists } = await checkInstitutionName(auth, name);
      if (exists) {
        notice.error('该机构名称已被使用,请更换名称');
        setNameAvailable(false);
      } else {
        notice.success('名称可用');
        setNameAvailable(true);
      }
    } catch (err) {
      notice.error(err, '');
      setNameAvailable(null);
    } finally {
      setNameChecking(false);
    }
  };

  const triggerLegalRepSearch = async () => {
    const q = (form.getFieldValue('legal_rep_sfid_number') ?? '').trim();
    if (!q) {
      notice.warning('请先输入法定代表人身份ID关键字');
      return;
    }
    setLegalRepSearching(true);
    try {
      const rows = await searchLegalRepresentativeCitizens(auth, q);
      setLegalRepOptions(rows);
      if (rows.length === 0) {
        notice.info('未找到正常状态公民');
      }
    } catch (err) {
      notice.error(err, '');
      setLegalRepOptions([]);
    } finally {
      setLegalRepSearching(false);
    }
  };

  const handlePhotoUpload = async (file: File) => {
    setPhotoUploading(true);
    try {
      const photo = await uploadLegalRepresentativePhoto(auth, file);
      form.setFieldsValue({
        legal_rep_photo_path: photo.file_path,
        legal_rep_photo_name: photo.file_name,
        legal_rep_photo_mime: photo.mime_type,
        legal_rep_photo_size: photo.file_size,
      });
      setPhotoName(photo.file_name);
      notice.success('证件照已上传');
    } catch (err) {
      notice.error(err, '证件照上传失败');
    } finally {
      setPhotoUploading(false);
    }
    return false;
  };

  const onSaveInfo = async (values: InfoFormValues) => {
    const name = values.institution_name.trim();
    if (!name) {
      notice.error('机构名称不能为空');
      return;
    }
    if (requiresParent && !values.parent_sfid_number) {
      notice.error('请选择所属法人机构');
      return;
    }
    const legalRepName = values.legal_rep_name?.trim();
    const legalRepSfid = values.legal_rep_sfid_number?.trim();
    if (!legalRepName || !legalRepSfid || !values.legal_rep_photo_path) {
      notice.error('请完整填写法定代表人姓名、身份ID和证件照');
      return;
    }
    // 名称变了必须查重通过才能保存
    if (!isNameUnchanged() && nameAvailable !== true) {
      notice.warning('请点击搜索图标检查名称是否可用');
      return;
    }
    setSavingInfo(true);
    try {
      await updateInstitution(auth, inst.sfid_number, {
        institution_name: name,
        parent_sfid_number: requiresParent ? values.parent_sfid_number : undefined,
        legal_rep_name: legalRepName,
        legal_rep_sfid_number: legalRepSfid,
        legal_rep_photo_path: values.legal_rep_photo_path,
        legal_rep_photo_name: values.legal_rep_photo_name,
        legal_rep_photo_mime: values.legal_rep_photo_mime,
        legal_rep_photo_size: values.legal_rep_photo_size,
      });
      notice.success('机构信息已保存');
      setEditing(false);
      onReload();
    } catch (err) {
      const raw = err instanceof Error ? err.message : '保存失败';
      if (raw.includes('已被使用') || raw.includes('同名机构')) {
        notice.error('该机构名称已被使用,请更换名称');
        setNameAvailable(false);
      } else {
        notice.error(err, '保存失败');
      }
    } finally {
      setSavingInfo(false);
    }
  };

  const titleText = inst.institution_name || '(未命名机构)';
  const createdByLabel = (() => {
    const roleLabel = detail.created_by_role
      ? CREATED_BY_ROLE_LABEL[detail.created_by_role] ?? detail.created_by_role
      : '';
    // 三态:姓名+角色 / 仅角色(内置管理员未设姓名)/ 完全未知
    if (detail.created_by_name) {
      return (
        <span>
          {detail.created_by_name}
          {roleLabel && (
            <Typography.Text type="secondary" style={{ marginLeft: 6, fontSize: 12 }}>
              ({roleLabel})
            </Typography.Text>
          )}
        </span>
      );
    }
    if (roleLabel) {
      return <span>{roleLabel}</span>;
    }
    return <span style={{ color: '#999' }}>未知</span>;
  })();

  // 保存按钮可用判断
  const saveEnabled = isNameUnchanged() || nameAvailable === true;

  // 右板块右上角按钮组
  const rightExtra = canWrite ? (
    !editing ? (
      <Button type="primary" onClick={onClickEdit}>
        编辑
      </Button>
    ) : (
      <Space>
        <Button onClick={onClickCancel}>取消</Button>
        <Button
          type="primary"
          loading={savingInfo}
          disabled={!saveEnabled}
          onClick={() => form.submit()}
          style={saveEnabled ? { backgroundColor: '#52c41a', borderColor: '#52c41a' } : undefined}
        >
          保存
        </Button>
      </Space>
    )
  ) : null;

  return (
    <>
      {/* 顶部:左右双板块;编辑/取消+保存 按钮挂在外层 Card 的 extra(机构名称右侧) */}
      <Card
        title={<span style={{ fontSize: 18, fontWeight: 600 }}>{titleText}</span>}
        extra={rightExtra}
        style={{ marginBottom: 16 }}
      >
        <Row gutter={24}>
          {/* 左:SFID 不可编辑身份信息 */}
          <Col xs={24} md={12}>
            <Descriptions column={1} size="small">
              <Descriptions.Item label="身份ID">
                <Typography.Text style={{ fontSize: 12, wordBreak: 'break-all' }}>
                  {inst.sfid_number}
                </Typography.Text>
              </Descriptions.Item>
              <Descriptions.Item label="省份">{inst.province}</Descriptions.Item>
              <Descriptions.Item label="城市">{inst.city}</Descriptions.Item>
              {/* 中文注释:p1/机构代码是系统编码,前端只显示中文;映射缺失时回退原代码兜底 */}
              <Descriptions.Item label="盈利属性">
                {inst.p1 === '0' ? '非盈利' : '盈利'}
              </Descriptions.Item>
              <Descriptions.Item label="机构">
                {INSTITUTION_CODE_LABEL[inst.institution_code] || inst.institution_code}
              </Descriptions.Item>
              <Descriptions.Item label="创建时间">
                {new Date(inst.created_at).toLocaleString('zh-CN')}
              </Descriptions.Item>
              <Descriptions.Item label="创建用户">{createdByLabel}</Descriptions.Item>
            </Descriptions>
          </Col>

          {/* 右:机构信息 — 直接展示 Form/Descriptions,不包额外 Card;按钮在外层 Card 的 extra */}
          <Col xs={24} md={12}>
            {needsCompletion && canWrite && !editing && (
              <Alert
                type="warning"
                showIcon
                message="请先完善机构名称和法定代表人资料,才能新建账户"
                style={{ marginBottom: 12 }}
              />
            )}

            {editing ? (
                <Form<InfoFormValues>
                  form={form}
                  layout="vertical"
                  onFinish={onSaveInfo}
                  initialValues={{
                    institution_name: inst.institution_name ?? '',
                    parent_sfid_number: inst.parent_sfid_number ?? undefined,
                    legal_rep_name: inst.legal_rep_name ?? '',
                    legal_rep_sfid_number: inst.legal_rep_sfid_number ?? '',
                    legal_rep_photo_path: inst.legal_rep_photo_path ?? '',
                    legal_rep_photo_name: inst.legal_rep_photo_name ?? '',
                    legal_rep_photo_mime: inst.legal_rep_photo_mime ?? '',
                    legal_rep_photo_size: inst.legal_rep_photo_size ?? undefined,
                  }}
                >
                  <Form.Item
                    label="机构名称"
                    name="institution_name"
                    rules={[
                      { required: true, message: '请输入机构名称' },
                      { max: 30, message: '最多 30 个字' },
                    ]}
                    extra={
                      isNameUnchanged()
                        ? '未修改名称,无需查重'
                        : nameAvailable === true
                          ? '名称可用'
                          : nameAvailable === false
                            ? '该名称已被占用,请更换'
                            : '修改后点击右侧搜索图标检查是否重名'
                    }
                  >
                    <Input
                      placeholder="请输入机构名称(最多 30 字)"
                      maxLength={30}
                      onChange={onNameInputChange}
                      suffix={
                        <span
                          style={{
                            cursor: nameChecking ? 'default' : 'pointer',
                            color: nameChecking ? '#999' : '#1890ff',
                          }}
                          onClick={nameChecking ? undefined : onCheckName}
                          title="检查名称是否重名"
                        >
                          {nameChecking ? <Spin size="small" /> : <SearchOutlined />}
                        </span>
                      }
                    />
                  </Form.Item>
                  {requiresParent && (
                    <Form.Item
                      label="所属法人"
                      name="parent_sfid_number"
                      rules={[{ required: true, message: '请选择所属法人机构' }]}
                      extra="输入 SFID 或机构名称后点击右侧搜索图标,从下拉结果中选择;必须是私法人(S)或公法人(G)"
                    >
                      <AutoComplete
                        // 不提供 onSearch → 用户输入时不自动请求,仅点搜索图标触发
                        filterOption={false}
                        notFoundContent={null}
                        options={parentSearchOpts.map((r) => ({
                          value: r.sfid_number,
                          label: (
                            <div>
                              <div style={{ fontWeight: 500 }}>{r.institution_name}</div>
                              <div style={{ fontSize: 11, color: '#888' }}>
                                {r.sfid_number} · {r.subject_property} · {r.province}/{r.city}
                              </div>
                            </div>
                          ),
                        }))}
                        onSelect={(val) => {
                          // 选中后,把选中机构缓存到 selectedParent 便于只读态展示
                          const hit = parentSearchOpts.find((o) => o.sfid_number === val);
                          if (hit) setSelectedParent(hit);
                        }}
                      >
                        <Input
                          placeholder="输入 SFID 或机构名称后点击右侧搜索图标"
                          suffix={
                            <span
                              style={{
                                cursor: parentSearching ? 'default' : 'pointer',
                                color: parentSearching ? '#999' : '#1890ff',
                              }}
                              onClick={triggerParentSearch}
                              title="搜索法人机构"
                            >
                              {parentSearching ? <Spin size="small" /> : <SearchOutlined />}
                            </span>
                          }
                        />
                      </AutoComplete>
                    </Form.Item>
                  )}
                  <Form.Item
                    label="法定代表人姓名"
                    name="legal_rep_name"
                    rules={[
                      { required: true, message: '请输入法定代表人姓名' },
                      { max: 30, message: '最多 30 个字' },
                    ]}
                  >
                    <Input placeholder="请输入法定代表人姓名" maxLength={30} />
                  </Form.Item>
                  <Form.Item
                    label="法定代表人身份ID"
                    name="legal_rep_sfid_number"
                    rules={[{ required: true, message: '请选择法定代表人身份ID' }]}
                  >
                    <AutoComplete
                      filterOption={false}
                      options={legalRepOptions.map((sfidNumber) => ({
                        value: sfidNumber,
                        label: sfidNumber,
                      }))}
                    >
                      <Input
                        placeholder="输入身份ID后点击搜索"
                        suffix={
                          <span
                            style={{
                              cursor: legalRepSearching ? 'default' : 'pointer',
                              color: legalRepSearching ? '#999' : '#1890ff',
                            }}
                            onClick={legalRepSearching ? undefined : triggerLegalRepSearch}
                            title="搜索正常状态公民"
                          >
                            {legalRepSearching ? <Spin size="small" /> : <SearchOutlined />}
                          </span>
                        }
                      />
                    </AutoComplete>
                  </Form.Item>
                  <Form.Item label="法定代表人证件照" required>
                    <Upload
                      accept="image/jpeg,image/png,image/webp"
                      showUploadList={false}
                      beforeUpload={(file) => handlePhotoUpload(file as File)}
                    >
                      <Button icon={<UploadOutlined />} loading={photoUploading}>
                        上传证件照
                      </Button>
                    </Upload>
                    {photoName && (
                      <div style={{ color: '#52c41a', marginTop: 8, fontSize: 12 }}>
                        {photoName}
                      </div>
                    )}
                  </Form.Item>
                  <Form.Item
                    name="legal_rep_photo_path"
                    rules={[{ required: true, message: '请上传法定代表人证件照' }]}
                    hidden
                  >
                    <Input />
                  </Form.Item>
                  <Form.Item name="legal_rep_photo_name" hidden><Input /></Form.Item>
                  <Form.Item name="legal_rep_photo_mime" hidden><Input /></Form.Item>
                  <Form.Item name="legal_rep_photo_size" hidden><Input type="number" /></Form.Item>
                </Form>
              ) : (
                // 只读展示
                <Descriptions column={1} size="small">
                  <Descriptions.Item label="机构名称">
                    {inst.institution_name || (
                      <span style={{ color: '#999' }}>(未命名)</span>
                    )}
                  </Descriptions.Item>
                  {requiresParent && (
                    <Descriptions.Item label="所属法人">
                      {inst.parent_sfid_number ? (
                        selectedParent ? (
                          <span>
                            {selectedParent.institution_name}
                            <Typography.Text
                              type="secondary"
                              style={{ marginLeft: 6, fontSize: 12 }}
                            >
                              ({selectedParent.sfid_number})
                            </Typography.Text>
                          </span>
                        ) : (
                          <Typography.Text code style={{ fontSize: 12 }}>
                            {inst.parent_sfid_number}
                          </Typography.Text>
                        )
                      ) : (
                        <span style={{ color: '#999' }}>(未设置)</span>
                      )}
                    </Descriptions.Item>
                  )}
                  {inst.private_type && (
                    <Descriptions.Item label="私权类型">
                      {PRIVATE_TYPE_LABEL[inst.private_type] || inst.private_type}
                    </Descriptions.Item>
                  )}
                  {inst.partnership_kind && (
                    <Descriptions.Item label="合伙类型">
                      {PARTNERSHIP_KIND_LABEL[inst.partnership_kind] || inst.partnership_kind}
                    </Descriptions.Item>
                  )}
                  {inst.has_legal_personality !== null &&
                    inst.has_legal_personality !== undefined && (
                    <Descriptions.Item label="法人资格">
                      <Tag color={inst.has_legal_personality ? 'green' : 'default'}>
                        {inst.has_legal_personality ? '有法人资格' : '无法人资格'}
                      </Tag>
                    </Descriptions.Item>
                  )}
                  <Descriptions.Item label="法定代表人姓名">
                    {inst.legal_rep_name || <span style={{ color: '#999' }}>(未填写)</span>}
                  </Descriptions.Item>
                  <Descriptions.Item label="法定代表人身份ID">
                    {inst.legal_rep_sfid_number ? (
                      <Typography.Text style={{ fontSize: 12, wordBreak: 'break-all' }}>
                        {inst.legal_rep_sfid_number}
                      </Typography.Text>
                    ) : (
                      <span style={{ color: '#999' }}>(未填写)</span>
                    )}
                  </Descriptions.Item>
                  <Descriptions.Item label="法定代表人证件照">
                    {inst.legal_rep_photo_name || <span style={{ color: '#999' }}>(未上传)</span>}
                  </Descriptions.Item>
                </Descriptions>
              )}
          </Col>
        </Row>
      </Card>

      {/* 中:账户列表 */}
      <Card
        type="inner"
        title={`账户列表(${accounts.length})`}
        extra={
          canWrite && (
            <Button
              type="primary"
              disabled={needsCompletion}
              title={needsCompletion ? '请先完善机构名称和法定代表人资料' : undefined}
              onClick={() => setCreateAccountOpen(true)}
            >
              + 新建账户
            </Button>
          )
        }
        style={{ marginBottom: 16 }}
      >
        <AccountList
          accounts={accounts}
          loading={loading}
          canDelete={canWrite}
          onDelete={onDeleteAccount}
        />
      </Card>

      {/* 下:资料库(自治模块) */}
      <DocumentLibrary
        auth={auth}
        sfidNumber={inst.sfid_number}
        canWrite={canWrite}
        createPasskeyChallengeGrant={createPasskeyChallengeGrant}
      />

      <CreateAccountModal
        auth={auth}
        sfidNumber={inst.sfid_number}
        institutionName={inst.institution_name ?? ''}
        existingAccounts={accounts}
        open={createAccountOpen}
        onCancel={() => setCreateAccountOpen(false)}
        onCreated={() => {
          setCreateAccountOpen(false);
          onReload();
        }}
      />
    </>
  );
};
