<script setup>
import { reactive, onMounted, ref, watch } from 'vue'
import { Message } from "@arco-design/web-vue";
import '@arco-design/web-vue/es/message/style/css.js'
import { invoke } from '@tauri-apps/api';
import * as echarts from 'echarts';

onMounted(() => {
    initRoomList()
})

const mainColor = '#3498db'
const defaultColor = '#bdc3c7'

const form = reactive({
    token: '', // 播酱 token
    url: '', // 页面 url（用来获取主播列表）
    mainId: '', // 主直播间房间号
})

function urlChange(e) {
    if (!e) return
    const search = new URL(e).search
    if (!search) return

    const rid = new URLSearchParams(search).get('rid')

    if (rid) {
        form.mainId = rid
    }
}

function handleSubmit(e) {
    if (!e.errors) {
        localStorage.setItem("config", JSON.stringify(e.values))
    }

    for (const key of Object.keys(e.values)) {
        config[key] = e.values[key]
    }

    Message.success("保存成功")
}


let config = reactive({})
// 加载表单历史数据
function init() {
    try {
        const newConfig = getConfig()
        if (newConfig) {
            for (const key of Object.keys(newConfig)) {
                form[key] = newConfig[key]
                config[key] = newConfig[key]
            }
        }
    } catch (e) {
        Message.error('加载历史数据失败')
    }
}
init()

const btnLoading = ref(false)
async function reload() {
    if (!Array.isArray(roomList.value) || !roomList.value.length) {
        Message.info({
            duration: 5000,
            content: "请先获取主播列表"
        })
        return
    }
    if (btnLoading.value) return

    btnLoading.value = true
    chartData.value = []
    getData()
}

async function getData() {
    if (!config.token || !config.url || !config.mainId) {
        Message.warning({
            duration: 5000,
            content: "请先配置信息并保存"
        })
    }

    // 不需要顺序，因为最后会排序
    // 先请求1个，如果需要登录，则直接退出并提示
    let res = await setDataById(roomList.value[0].id)
    if (!res) return

    // 之后每次请求3个
    let index = 1
    const max = roomList.value.length - 1
    if (max <= index) return
    let timer = setInterval(() => {
        index = processRoomData(index);
        if (index > max) {
            clearInterval(timer);
            timer = null;
        }
    }, 1500);
}

function processRoomData(startIndex) {
    const count = 3 // 每次取的个数
    const max = roomList.value.length - 1
    const endIndex = Math.min(startIndex + count, max + 1); // 计算结束索引，避免越界

    for (let i = startIndex; i < endIndex; i++) {
        setDataById(roomList.value[i].id);
    }

    return endIndex;
}

async function setDataById(id) {
    const date = getTodayDate()

    let firstRes = await invoke('get_user_data', {
        date,
        rid: id,
        token: form.token
    })
    const parseData = JSON.parse(firstRes)
    // console.log('parseData', parseData)

    if (parseData.code !== 0) {
        Message.error({
            duration: 5000,
            content: "数据获取失败，请重新设置token或联系管理员"
        })
        btnLoading.value = false
        return false
    }

    chartData.value.push(parseData.data)

    // 如果数据个数和主播个数相等，表示加载完成
    if (chartData.value.length === roomList.value.length) {
        btnLoading.value = false
        render(selectItem.value)
    }

    return true
}

// 获取表单配置
function getConfig() {
    return JSON.parse(localStorage.getItem("config"))
}


const selectList = ref([
    {
        name: '付费礼物',
        key: 'gift_new_yc'
    },
    {
        name: '礼物总值',
        key: 'yc_gift_value'
    },
    {
        name: '弹幕数量',
        key: 'danmu_count'
    },
    {
        name: '弹幕人数',
        key: 'danmu_person_count'
    },
    {
        name: '平均热度',
        key: 'hn_avg'
    },
    {
        name: '峰值热度',
        key: 'hn_max'
    },
    {
        name: '活跃观众',
        key: 'audience_count'
    },
    {
        name: '送礼人数',
        key: 'gift_person_count'
    },
])
const selectItem = ref('gift_new_yc')

// 每次重新选择后，刷新图表
watch(selectItem, (selectItem) => {
    render(selectItem)
})

let chartData = ref([])

let chartContainer = null
function render(key) {
    if (!chartContainer) {
        chartContainer = echarts.init(document.querySelector('#center-chart'))
    }
    // 先排好序
    const sortList = chartData.value.sort((a, b) => b.total_statistic[key] - a.total_statistic[key])

    if (!Array.isArray(sortList) || !sortList.length) {
        Message.info("请先获取数据")
        return
    }

    const options = {
        xAxis: {
            type: 'category',
            data: sortList.map(item => item.name || '--'),
            axisLabel: {
                show: true,
                interval: 0,
            }
        },
        yAxis: {
            type: 'value'
        },
        series: [
            {
                data: sortList.map(item => {
                    return {
                        value: item.total_statistic[key],
                        itemStyle: {
                            color: Number(item.rid) === Number(config.mainId) ? mainColor : defaultColor
                        }
                    }
                }),
                type: 'bar',
                label: {
                    show: true,
                    position: 'top'
                },
                markLine: {
                    data: [{
                        name: '平均线',
                        type: 'average',
                    }],
                    lineStyle: {
                        color: '#e67e22'
                    }
                }
            }
        ]
    };

    chartContainer.setOption(options)
}

async function getRoomList() {
    if (!config.url) return
    let result = []

    const res = await invoke("get_dy_loadpage_url", {
        url: config.url
    })
    // console.log('res', res)
    const parseData = JSON.parse(res)

    if (parseData.code === 0) {
        let list = parseData.data.page_config.components
        let childItem = list.find(item => item.children)
        if (!childItem) return
        let arr = childItem.children[0]
        while (arr.children) {
            let idx = arr.children.findIndex(item => item.children)
            idx < 0 && (idx = 0)
            if (arr.children[idx].name === "NewPcBasicRoomButton") {
                break
            }
            arr = arr.children[idx]
        }

        // 主播列表
        if (Array.isArray(arr.children)) {
            result = arr.children.map(item => {
                return {
                    name: item.props.text,
                    id: item.props.onlineRoomId
                }
            })
        }
    }

    return result
}


const roomList = ref([])

async function initRoomList() {
    let list = JSON.parse(localStorage.getItem("roomList"))
    if (!Array.isArray(list) || !list.length) {
        return reloadAnchor()
    }

    roomList.value = list
}

const anchorLoading = ref(false)
// 获取后储存，并返回
async function reloadAnchor() {
    anchorLoading.value = true
    const list = await getRoomList()
    // console.log("list", list)
    if (Array.isArray(list) && list.length) {
        localStorage.setItem("roomList", JSON.stringify(list))

        roomList.value = list
    }
    anchorLoading.value = false
}

const dateValue = ref(Date.now())

function dateChange(e) {
    console.log(e)
}

function getTodayDate() {
    const currentDate = new Date();

    const year = currentDate.getFullYear();

    // 获取月份（月份从0开始，所以需要加1）
    let month = currentDate.getMonth() + 1;
    month = month < 10 ? '0' + month : month; // 补0

    // 获取日期
    let day = currentDate.getDate();
    day = day < 10 ? '0' + day : day; // 补0

    // 格式化日期字符串
    return year + '-' + month + '-' + day;
}
</script>

<template>
    <div class="container">
        <a-collapse :default-active-key="['1', 2]">
            <a-collapse-item header="配置" key="1">
                <a-form :model="form" :style="{ width: '600px' }" @submit="handleSubmit">
                    <a-form-item field="token" label="token(播酱)" required>
                        <a-input v-model="form.token" placeholder="请输入token" />
                    </a-form-item>
                    <a-form-item field="url" label="url" required>
                        <a-input @change="urlChange" v-model="form.url" placeholder="请输入url" />
                    </a-form-item>
                    <a-form-item field="mainId" label="主直播间id" required>
                        <a-input v-model="form.mainId" placeholder="请输入id" />
                    </a-form-item>
                    <a-form-item>
                        <a-button type="primary" html-type="submit">保存</a-button>
                    </a-form-item>
                </a-form>
            </a-collapse-item>
        </a-collapse>

        <div class="anchor-list">
            <div class="tool">
                <div class="subtitle">主播列表</div>
                <a-button :loading="anchorLoading" @click="reloadAnchor" class="reload-anchor">刷新</a-button>
            </div>
            <a-tag :color="item.id === config.mainId ? mainColor : defaultColor" class="anchor-item"
                v-for="item in roomList" :key="item.id">{{ item.name }}</a-tag>
        </div>

        <div class="view">
            <div class="tool">
                <div class="subtitle">浏览</div>
                <a-button @click="reload" :loading="btnLoading">刷新</a-button>
            </div>

            <div class="center">
                <div>
                    <a-select :style="{ width: '220px', marginBottom: '10px' }" v-model="selectItem">
                        <a-option v-for="item in selectList" :value="item.key" :key="item.key">{{ item.name
                            }}</a-option>
                    </a-select>
                </div>

                <!-- <a-date-picker @change="dateChange" v-model="dateValue" style="width: 200px;" /> -->
                <div id="center-chart" class="chart"></div>
            </div>
        </div>
    </div>
</template>

<style scoped>
.container {
    padding: 30px;
}

.view {
    margin: 20px 0;
}

.subtitle {
    font-size: 18px;
}

.tool {
    display: flex;
    justify-content: space-between;
    margin: 15px 0;
}

.center {
    /* display: flex;
    flex-flow: row nowrap; */
}

.chart {
    width: 100%;
    height: 500px;
}

.anchor-item {
    margin: 0 5px 5px 0;
}
</style>