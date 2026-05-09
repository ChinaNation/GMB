// 其他 tab 内容类型，对齐后端 src/other/other-tabs。

type OtherDocumentTabItem = {
  key: 'whitepaper' | 'constitution';
  title: string;
  contentType: 'document';
};

type OtherTextTabItem = {
  key: string;
  title: string;
  contentType: 'text';
  text: string;
};

export type OtherTabItem = OtherDocumentTabItem | OtherTextTabItem;

export type OtherTabsPayload = {
  tabs: OtherTabItem[];
};
