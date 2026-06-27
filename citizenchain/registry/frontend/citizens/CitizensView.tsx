// 中文注释:注册局顶层视图 —— activeView === 'citizens' 分支。
// 包含:citizen 列表 + 搜索栏 + 表格 + 直接录入公民弹窗。

import { useEffect, useState } from 'react';
import { Button, Card, Descriptions, Form, Input, Modal, Space, Table, Tag } from 'antd';
import { SearchOutlined } from '@ant-design/icons';

import type { ColumnsType } from 'antd/es/table';
import {
  listCitizens,
  type CitizenRow,
} from './api';
import { decodeSs58, tryEncodeSs58 } from '../utils/ss58';
import { useAuth } from '../hooks/useAuth';
import { glassCardStyle, glassCardHeadStyle } from '../core/cardStyles';
import { CitizenCreateModal } from './CitizenCreateModal';
import { notice } from '../utils/notice';


export function CitizensView() {
  const { auth, capabilities } = useAuth();
  const [searchForm] = Form.useForm<{ keyword: string }>();
  const [rows, setRows] = useState<CitizenRow[]>([]);
  const [loading, setLoading] = useState(false);
  const [searchKeyword, setSearchKeyword] = useState('');
  const [nextCursor, setNextCursor] = useState<string | null>(null);
  const [cursorStack, setCursorStack] = useState<string[]>([]);

  // 直接录入弹窗控制
  const [createModalOpen, setCreateModalOpen] = useState(false);
  const [detailRecord, setDetailRecord] = useState<CitizenRow | null>(null);

  const refreshList = async (keyword: string, cursor?: string | null, silent?: boolean) => {
    if (!auth) return;
    const exact = keyword.trim();
    if (!exact) {
      setRows([]);
      setNextCursor(null);
      return;
    }
    setLoading(true);
    try {
      const raw = await listCitizens(auth, exact, cursor);
      const list = raw.items;
      setRows(list);
      setNextCursor(raw.next_cursor ?? null);
      if (list.length === 0) {
        notice.warningModal({
          title: '查询结果',
          content: '未查询到公民信息',
        });
      }
    } catch (err) {
      if (!silent) {
        notice.error(err, '查询失败');
      }
    } finally {
      setLoading(false);
    }
  };

  // 挂载时自动加载;auth 变化时(登录/登出)重新加载
  useEffect(() => {
    if (!auth) {
      setRows([]);
      setSearchKeyword('');
      setNextCursor(null);
      setCursorStack([]);
      setCreateModalOpen(false);
      return;
    }
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [auth]);

  const onSearch = async (values: { keyword: string }) => {
    if (!auth) return;
    let keyword = values.keyword?.trim() || '';
    if (keyword) {
      try {
        keyword = decodeSs58(keyword);
      } catch {
        // 非 SS58 格式,保留原值
      }
    }
    setSearchKeyword(keyword);
    setCursorStack([]);
    await refreshList(keyword);
  };

  const onNextPage = async () => {
    if (!nextCursor || !searchKeyword) return;
    setCursorStack((prev) => [...prev, nextCursor]);
    await refreshList(searchKeyword, nextCursor, true);
  };

  const onPrevPage = async () => {
    if (!searchKeyword || cursorStack.length === 0) return;
    const stack = [...cursorStack];
    stack.pop();
    const prevCursor = stack.length > 0 ? stack[stack.length - 1] : null;
    setCursorStack(stack);
    await refreshList(searchKeyword, prevCursor, true);
  };

  // 中文注释：录入成功后，用返回的新身份ID自动回填搜索框并查询，让新公民立即显示在列表。
  // 拿不到新号(理论不应发生)时回退到沿用当前关键字刷新。
  const handleCreated = async (createdCid?: string) => {
    const next = createdCid?.trim();
    if (next) {
      searchForm.setFieldsValue({ keyword: next });
      setSearchKeyword(next);
      setCursorStack([]);
      await refreshList(next, null, true);
      return;
    }
    await refreshList(searchKeyword, null, true);
  };

  const bindStatusText = (v: string | undefined) => {
    if (v === 'PENDING') return '待绑定';
    if (v === 'BOUND') return '已绑定';
    return v ?? '-';
  };

  const statusTag = (status: string | undefined) => (
    status === 'NORMAL' ? <Tag color="green">正常</Tag> : <Tag color="red">注销</Tag>
  );

  const citizenStatusText = (status: string | undefined) => {
    if (status === 'NORMAL') return '正常';
    if (status === 'REVOKED') return '注销';
    return '-';
  };

  const formatDateRange = (from?: string, until?: string) => {
    if (!from || !until) return '-';
    return `${formatDate(from)}-${formatDate(until)}`;
  };

  const formatDate = (value: string) => {
    const parts = value.split('-');
    if (parts.length !== 3) return value;
    return `${parts[0]}年${parts[1]}月${parts[2]}日`;
  };

  const electionRangesText = (
    scope: CitizenRow['election_scope_level'],
    provinceName?: string,
    cityName?: string,
    townName?: string,
  ) => {
    const ranges = ['全国选举公民'];
    if (provinceName?.trim()) ranges.push(`${provinceName}选举公民`);
    if ((scope === 'CITY' || scope === 'TOWN') && cityName?.trim()) {
      ranges.push(`${cityName}选举公民`);
    }
    if (scope === 'TOWN' && townName?.trim()) {
      ranges.push(`${townName}选举公民`);
    }
    return ranges.join('、');
  };

  const citizenColumns: ColumnsType<CitizenRow> = [
    {
      title: '序号',
      width: 80,
      align: 'center',
      render: (_v: unknown, _r: CitizenRow, idx: number) => idx + 1,
    },
    {
      title: '投票账户',
      dataIndex: 'wallet_address',
      align: 'center',
      render: (v: string | undefined) => v ?? '-',
    },
    {
      title: '身份ID',
      dataIndex: 'cid_number',
      align: 'center',
      render: (v: string | undefined) => v ?? '-',
    },
    {
      title: '投票状态',
      dataIndex: 'vote_status',
      width: 120,
      align: 'center',
      render: (v: string | undefined) => statusTag(v),
    },
  ];
  return (
    <>
      <Card
        title={'公民身份列表'}
        bordered={false}
        style={glassCardStyle}
        headStyle={glassCardHeadStyle}
        extra={
          <Space>
            <Form form={searchForm} layout="inline" onFinish={onSearch}>
              <Form.Item name="keyword" style={{ marginBottom: 0 }}>
                <Input
                  style={{ width: 420 }}
                  placeholder="请输入身份ID或投票账户检索"
                  allowClear
                  onPressEnter={() => searchForm.submit()}
                  suffix={
                    <SearchOutlined
                      onClick={() => searchForm.submit()}
                      style={{ color: loading ? '#999' : '#1677ff', cursor: 'pointer' }}
                    />
                  }
                />
              </Form.Item>
              {capabilities.canBusinessWrite && (
                <Form.Item style={{ marginBottom: 0 }}>
                  <Button type="primary" onClick={() => setCreateModalOpen(true)}>
                    新增公民
                  </Button>
                </Form.Item>
              )}
            </Form>
          </Space>
        }
      >
        <Table<CitizenRow>
          rowKey={(r) => `${r.id}`}
          dataSource={rows}
          loading={loading}
          pagination={false}
          columns={citizenColumns}
          onRow={(record) => ({
            onClick: (event) => {
              // 中文注释：操作栏是独立交互区，点击绑定按钮时不能再触发行详情弹窗。
              const target = event.target as EventTarget | null;
              if (target instanceof Element && target.closest('[data-row-action="true"]')) return;
              setDetailRecord(record);
            },
            style: { cursor: 'pointer' },
          })}
        />
        <Space style={{ marginTop: 12 }}>
          <Button disabled={loading || cursorStack.length === 0} onClick={onPrevPage}>
            上一页
          </Button>
          <Button disabled={loading || !nextCursor} onClick={onNextPage}>
            下一页
          </Button>
        </Space>
      </Card>

      <Modal
        title="公民信息详情"
        open={!!detailRecord}
        footer={null}
        onCancel={() => setDetailRecord(null)}
        destroyOnClose
        width={720}
      >
        {detailRecord && (
          <Descriptions column={1} size="small" bordered>
            <Descriptions.Item label="身份ID">{detailRecord.cid_number ?? '-'}</Descriptions.Item>
            <Descriptions.Item label="投票账户">
              {/* 中文注释:wallet_address 缺失时把公钥转 SS58 兜底,前端不显示裸公钥 */}
              {detailRecord.wallet_address
                ?? (detailRecord.wallet_pubkey ? tryEncodeSs58(detailRecord.wallet_pubkey) || '-' : '-')}
            </Descriptions.Item>
            <Descriptions.Item label="绑定状态">{bindStatusText(detailRecord.bind_status)}</Descriptions.Item>
            <Descriptions.Item label="选举权利">{detailRecord.voting_eligible ? '有' : '无'}</Descriptions.Item>
            <Descriptions.Item label="公民状态">{citizenStatusText(detailRecord.citizen_status)}</Descriptions.Item>
            <Descriptions.Item label="投票范围">
              {electionRangesText(
                detailRecord.election_scope_level,
                detailRecord.residence_province_name,
                detailRecord.residence_city_name,
                detailRecord.residence_town_name,
              )}
            </Descriptions.Item>
            <Descriptions.Item label="参选范围">
              {electionRangesText(
                detailRecord.election_scope_level,
                detailRecord.birth_province_name,
                detailRecord.birth_city_name,
                detailRecord.birth_town_name,
              )}
            </Descriptions.Item>
            <Descriptions.Item label="有效期">
              {formatDateRange(detailRecord.valid_from, detailRecord.valid_until)}
            </Descriptions.Item>
          </Descriptions>
        )}
      </Modal>

      {capabilities.canBusinessWrite && (
        <CitizenCreateModal
          auth={auth}
          open={createModalOpen}
          onClose={() => setCreateModalOpen(false)}
          onCreated={handleCreated}
        />
      )}
    </>
  );
}
