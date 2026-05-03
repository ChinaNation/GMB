// 中文注释:私权机构详情页布局 — 顶部左右双板块 + 账户 + 资料库。
//
// 布局:
//   顶部一整块 Card(标题 = 机构名称,编辑/取消/保存按钮在 Card extra 右上角):
//     ┌ 左 Col:SFID 信息(只读)──────────────┐  ┌ 右 Col:机构信息 ──────────────┐
//     │ SFID / 省 / 市 / A3 / P1 / 机构代码 │  │ 机构名称 + 搜索查重图标       │
//     │ 创建时间 / 创建用户                  │  │ 企业类型 Select(仅 SFR)       │
//     │                                      │  │ 所属法人 AutoComplete(仅 FFR) │
//     └──────────────────────────────────────┘  └───────────────────────────────┘
//
// 右板块交互:
//   默认态 = 只读 Descriptions 展示,右上角显示"编辑"按钮
//   编辑态 = Form 可操作,右上角切换为"取消" + "保存"
//   机构名称右侧搜索图标:输入后点击查重;重名则禁止保存;名称未改动视为已通过
//   FFR 所属法人:输入后点搜索图标触发模糊搜索(/institution/search-parents)
//
// 账户列表(AccountList)每家机构自带"主账户"/"费用账户"两条默认账户,
// 创建后只登记在 SFID;链上状态由区块链软件同步回来。

import React, { useEffect, useMemo, useState } from 'react';
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
  Select,
  Space,
  Spin,
  Tag,
  Typography,
  message,
} from 'antd';
import { SearchOutlined } from '@ant-design/icons';
import type { AdminAuth } from '../auth/types';
import {
  A3_LABEL,
  INSTITUTION_CODE_LABEL,
  SUB_TYPE_LABEL,
  subTypeChoicesForP1,
} from './locks';
import {
  checkInstitutionName,
  searchParentInstitutions,
  updateInstitution,
  type InstitutionDetail,
  type ParentInstitutionRow,
} from './api';
import { AccountList } from './AccountList';
import { CreateAccountModal } from './CreateAccountModal';
import { DocumentLibrary } from './DocumentLibrary';
import {
  CLEARING_BANK_ELIGIBLE_LABEL,
  isClearingBankEligible,
} from './utils/clearingBankEligible';

// 创建者角色中文映射(与列表页保持一致)。
const CREATED_BY_ROLE_LABEL: Record<string, string> = {
  SHENG_ADMIN: '省级管理员',
  SHI_ADMIN: '市级管理员',
};

const INSTITUTION_CHAIN_STATUS_LABEL: Record<string, string> = {
  NOT_REGISTERED: '未上链',
  PENDING_REGISTER: '注册中',
  REGISTERED: '已上链',
  REVOKED_ON_CHAIN: '已注销',
};

interface Props {
  auth: AdminAuth;
  detail: InstitutionDetail;
  canWrite: boolean;
  loading: boolean;
  onReload: () => void;
  onDeleteAccount: (accountName: string) => void;
}

interface InfoFormValues {
  institution_name: string;
  sub_type?: string;
  /** 非法人(FFR)所属法人 sfid_id */
  parent_sfid_id?: string;
}

export const PrivateInstitutionLayout: React.FC<Props> = ({
  auth,
  detail,
  canWrite,
  loading,
  onReload,
  onDeleteAccount,
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

  const isSFR = inst.a3 === 'SFR';
  const isFFR = inst.a3 === 'FFR';

  const subTypeChoices = useMemo(
    () => (isSFR ? subTypeChoicesForP1(inst.p1) : []),
    [isSFR, inst.p1],
  );
  // 完善判断:名称必填;SFR 需要 sub_type;FFR 需要 parent_sfid_id
  const needsCompletion =
    !inst.institution_name ||
    (isSFR && !inst.sub_type) ||
    (isFFR && !inst.parent_sfid_id);

  // ── FFR 所属法人搜索 ──
  const [parentSearchOpts, setParentSearchOpts] = useState<ParentInstitutionRow[]>([]);
  const [parentSearching, setParentSearching] = useState(false);
  // 当前选中的法人(用于展示已选项名称;首次进入若 inst.parent_sfid_id 有值,也要一次性拿到显示名)
  const [selectedParent, setSelectedParent] = useState<ParentInstitutionRow | null>(null);

  // detail 变更 → 若有 parent_sfid_id 则拉一次展示名称
  useEffect(() => {
    if (!isFFR || !inst.parent_sfid_id) {
      setSelectedParent(null);
      return;
    }
    // 用 sfid_id 自身作为查询词反查名称
    let cancelled = false;
    searchParentInstitutions(auth, inst.parent_sfid_id)
      .then((rows) => {
        if (cancelled) return;
        const hit = rows.find((r) => r.sfid_id === inst.parent_sfid_id);
        setSelectedParent(hit ?? null);
      })
      .catch(() => {
        if (!cancelled) setSelectedParent(null);
      });
    return () => {
      cancelled = true;
    };
  }, [isFFR, inst.parent_sfid_id, auth.access_token]);

  // 搜索(仅在用户点击搜索图标时触发,不自动 onSearch)
  const onParentSearch = async (value: string) => {
    const q = value.trim();
    if (!q) {
      message.warning('请先输入 SFID 或机构名称');
      setParentSearchOpts([]);
      return;
    }
    setParentSearching(true);
    try {
      const rows = await searchParentInstitutions(auth, q);
      setParentSearchOpts(rows);
      if (rows.length === 0) {
        message.info('未找到匹配的法人机构');
      }
    } catch (err) {
      message.error(err instanceof Error ? err.message : '搜索失败');
      setParentSearchOpts([]);
    } finally {
      setParentSearching(false);
    }
  };

  const triggerParentSearch = () => {
    if (parentSearching) return;
    const q = (form.getFieldValue('parent_sfid_id') ?? '') as string;
    onParentSearch(q);
  };

  // detail 重新加载(保存成功后 onReload)→ 重置编辑态
  useEffect(() => {
    setEditing(false);
    setNameAvailable(null);
    setCurrentName(inst.institution_name ?? '');
    form.setFieldsValue({
      institution_name: inst.institution_name ?? '',
      sub_type: inst.sub_type ?? undefined,
      parent_sfid_id: inst.parent_sfid_id ?? undefined,
    });
  }, [inst.sfid_id, inst.institution_name, inst.sub_type]);

  const onClickEdit = () => {
    setEditing(true);
    setNameAvailable(null);
    form.setFieldsValue({
      institution_name: inst.institution_name ?? '',
      sub_type: inst.sub_type ?? undefined,
      parent_sfid_id: inst.parent_sfid_id ?? undefined,
    });
    setCurrentName(inst.institution_name ?? '');
  };

  const onClickCancel = () => {
    setEditing(false);
    setNameAvailable(null);
    form.setFieldsValue({
      institution_name: inst.institution_name ?? '',
      sub_type: inst.sub_type ?? undefined,
      parent_sfid_id: inst.parent_sfid_id ?? undefined,
    });
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
      message.warning('请先输入机构名称');
      return;
    }
    if (isNameUnchanged()) {
      // 与原名一致,直接视为可用
      setNameAvailable(true);
      return;
    }
    setNameChecking(true);
    try {
      // 私权机构全国唯一查重(不传 a3/city 即走全国范围;后端会排除自身名不在此函数,
      // 所以必须在名称改动时才调用;未改名的场景已在 isNameUnchanged 提前返回)
      const { exists } = await checkInstitutionName(auth, name);
      if (exists) {
        message.error('该机构名称已被使用,请更换名称');
        setNameAvailable(false);
      } else {
        message.success('名称可用');
        setNameAvailable(true);
      }
    } catch (err) {
      message.error(err instanceof Error ? err.message : '查重失败');
      setNameAvailable(null);
    } finally {
      setNameChecking(false);
    }
  };

  const onSaveInfo = async (values: InfoFormValues) => {
    const name = values.institution_name.trim();
    if (!name) {
      message.error('机构名称不能为空');
      return;
    }
    if (isSFR && !values.sub_type) {
      message.error('请选择企业类型');
      return;
    }
    if (isFFR && !values.parent_sfid_id) {
      message.error('请选择所属法人机构');
      return;
    }
    // 名称变了必须查重通过才能保存
    if (!isNameUnchanged() && nameAvailable !== true) {
      message.warning('请点击搜索图标检查名称是否可用');
      return;
    }
    setSavingInfo(true);
    try {
      await updateInstitution(auth, inst.sfid_id, {
        institution_name: name,
        sub_type: isSFR ? values.sub_type ?? null : null,
        parent_sfid_id: isFFR ? values.parent_sfid_id : undefined,
      });
      message.success('机构信息已保存');
      setEditing(false);
      onReload();
    } catch (err) {
      const raw = err instanceof Error ? err.message : '保存失败';
      if (raw.includes('已被使用') || raw.includes('同名机构')) {
        message.error('该机构名称已被使用,请更换名称');
        setNameAvailable(false);
      } else {
        message.error(raw);
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
              <Descriptions.Item label="机构 SFID">
                <Typography.Text code style={{ fontSize: 12, wordBreak: 'break-all' }}>
                  {inst.sfid_id}
                </Typography.Text>
              </Descriptions.Item>
              <Descriptions.Item label="省份">{inst.province}</Descriptions.Item>
              <Descriptions.Item label="城市">{inst.city}</Descriptions.Item>
              <Descriptions.Item label="A3 类型">
                {inst.a3}/{A3_LABEL[inst.a3] || inst.a3}
              </Descriptions.Item>
              <Descriptions.Item label="P1 盈利属性">
                {inst.p1}/{inst.p1 === '0' ? '非盈利' : '盈利'}
              </Descriptions.Item>
              <Descriptions.Item label="机构代码">
                {inst.institution_code}/{INSTITUTION_CODE_LABEL[inst.institution_code] || inst.institution_code}
              </Descriptions.Item>
              <Descriptions.Item label="链上状态">
                <Tag>{INSTITUTION_CHAIN_STATUS_LABEL[inst.chain_status] || inst.chain_status}</Tag>
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
                message="请先完善机构名称与企业类型,才能新建账户"
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
                    sub_type: inst.sub_type ?? undefined,
                    parent_sfid_id: inst.parent_sfid_id ?? undefined,
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
                  {isFFR && (
                    <Form.Item
                      label="所属法人"
                      name="parent_sfid_id"
                      rules={[{ required: true, message: '请选择所属法人机构' }]}
                      extra="输入 SFID 或机构名称后点击右侧搜索图标,从下拉结果中选择;必须是私法人(SFR)或公法人(GFR)"
                    >
                      <AutoComplete
                        // 不提供 onSearch → 用户输入时不自动请求,仅点搜索图标触发
                        filterOption={false}
                        notFoundContent={null}
                        options={parentSearchOpts.map((r) => ({
                          value: r.sfid_id,
                          label: (
                            <div>
                              <div style={{ fontWeight: 500 }}>{r.institution_name}</div>
                              <div style={{ fontSize: 11, color: '#888' }}>
                                {r.sfid_id} · {r.a3} · {r.province}/{r.city}
                              </div>
                            </div>
                          ),
                        }))}
                        onSelect={(val) => {
                          // 选中后,把选中机构缓存到 selectedParent 便于只读态展示
                          const hit = parentSearchOpts.find((o) => o.sfid_id === val);
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
                  {isSFR && (
                    <Form.Item
                      label="企业类型"
                      name="sub_type"
                      rules={[{ required: true, message: '请选择企业类型' }]}
                      extra={
                        // 资格白名单提示(2026-04-24, ADR-007):
                        // 选中 JOINT_STOCK 时额外提示"可参与清算业务",其他类型保留原文案。
                        inst.p1 === '0'
                          ? '当前 P1=非盈利,企业类型锁定为公益组织'
                          : '当前 P1=盈利,可选四种企业类型;选择"股份公司"可参与清算业务(在区块链节点软件中注册为清算行)'
                      }
                    >
                      <Select options={subTypeChoices} placeholder="请选择企业类型" />
                    </Form.Item>
                  )}
                </Form>
              ) : (
                // 只读展示
                <Descriptions column={1} size="small">
                  <Descriptions.Item label="机构名称">
                    {inst.institution_name || (
                      <span style={{ color: '#999' }}>(未命名)</span>
                    )}
                  </Descriptions.Item>
                  {isFFR && (
                    <Descriptions.Item label="所属法人">
                      {inst.parent_sfid_id ? (
                        selectedParent ? (
                          <span>
                            {selectedParent.institution_name}
                            <Typography.Text
                              type="secondary"
                              style={{ marginLeft: 6, fontSize: 12 }}
                            >
                              ({selectedParent.sfid_id})
                            </Typography.Text>
                          </span>
                        ) : (
                          <Typography.Text code style={{ fontSize: 12 }}>
                            {inst.parent_sfid_id}
                          </Typography.Text>
                        )
                      ) : (
                        <span style={{ color: '#999' }}>(未设置)</span>
                      )}
                    </Descriptions.Item>
                  )}
                  {isSFR && (
                    <Descriptions.Item label="企业类型">
                      {inst.sub_type ? (
                        <>
                          {SUB_TYPE_LABEL[inst.sub_type] || inst.sub_type}
                          {/* 清算行资格 badge(2026-04-24, ADR-007):
                              SFR + JOINT_STOCK 自身判定为可作为清算行;
                              不需要 parent 信息 */}
                          {isClearingBankEligible(inst, null) && (
                            <Tag color="blue" style={{ marginLeft: 8 }}>
                              {CLEARING_BANK_ELIGIBLE_LABEL}
                            </Tag>
                          )}
                        </>
                      ) : (
                        <span style={{ color: '#999' }}>(未设置)</span>
                      )}
                    </Descriptions.Item>
                  )}
                  {/* FFR 资格 badge(2026-04-24, ADR-007):
                      需要 parent 信息(parent.SFR + parent.JOINT_STOCK)。
                      selectedParent 已在 useEffect 里按 parent_sfid_id 反查并缓存。 */}
                  {isFFR && selectedParent && isClearingBankEligible(inst, selectedParent) && (
                    <Descriptions.Item label="清算行资格">
                      <Tag color="blue">{CLEARING_BANK_ELIGIBLE_LABEL}</Tag>
                    </Descriptions.Item>
                  )}
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
              title={needsCompletion ? '请先完善机构名称与企业类型' : undefined}
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
      <DocumentLibrary auth={auth} sfidId={inst.sfid_id} canWrite={canWrite} />

      <CreateAccountModal
        auth={auth}
        sfidId={inst.sfid_id}
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
